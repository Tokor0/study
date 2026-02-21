use crate::config::{
    GlobalConfig, find_course_dir, find_latest_exercise, load_course_config, load_state,
    save_state,
};
use std::io::{self, Write};
use std::process::Command;

pub fn run(
    global: &GlobalConfig,
    course_name: Option<&str>,
    exercise_name: Option<&str>,
) -> io::Result<()> {
    let mut state = load_state();

    let course_input = course_name
        .or(state.last_course.as_deref())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "No course specified and no previous session found.",
            )
        })?
        .to_string();

    let course_dir = find_course_dir(global, &course_input).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Course not found: {}", course_input),
        )
    })?;

    let config = load_course_config(&course_dir)?;
    let course_code = &config.course.code;
    let display_name = config
        .course
        .name
        .as_deref()
        .unwrap_or(course_code)
        .to_string();

    let exercise = exercise_name
        .map(String::from)
        .or_else(|| state.last_exercises.get(course_code).cloned())
        .or_else(|| find_latest_exercise(&course_dir));

    // If no exercise was resolved and the course supports exercises, prompt to create one
    let exercise = match exercise {
        Some(ex) => Some(ex),
        None if !config.exercise_types.is_empty() => {
            prompt_create_exercise(global, &course_input, &display_name)?;
            state = load_state();
            state
                .last_exercises
                .get(course_code)
                .cloned()
                .or_else(|| find_latest_exercise(&course_dir))
        }
        None => None,
    };

    let work_dir = exercise
        .as_deref()
        .map(|ex| course_dir.join(ex))
        .filter(|dir| dir.is_dir())
        .unwrap_or_else(|| course_dir.clone());

    if config.study.commands.is_empty() {
        println!("No study commands configured for '{}'.", course_input);
        return Ok(());
    }

    match exercise.as_deref() {
        Some(ex) if work_dir != course_dir => {
            println!(
                "Starting study session for '{}' / '{}'...",
                display_name, ex
            );
        }
        _ => {
            println!("Starting study session for '{}'...", display_name);
        }
    }

    for cmd in &config.study.commands {
        println!("  Running: {}", cmd);
        if let Err(e) = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(&work_dir)
            .spawn()
        {
            eprintln!("  Failed to run '{}': {}", cmd, e);
        }
    }

    state.last_course = Some(course_code.clone());
    if let Some(ex) = exercise {
        state.last_exercises.insert(course_code.clone(), ex);
    }
    save_state(&state)?;

    Ok(())
}

fn prompt_create_exercise(
    global: &GlobalConfig,
    course_input: &str,
    display_name: &str,
) -> io::Result<()> {
    print!(
        "No exercises found for '{}'. Create one? [Y/n] ",
        display_name
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let answer = input.trim().to_lowercase();
    if !answer.is_empty() && answer != "y" && answer != "yes" {
        return Ok(());
    }

    crate::commands::init_exercise::run(global, None, None, Some(course_input))
}
