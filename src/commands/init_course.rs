use crate::config::{
    CourseConfig, CourseInfo, GlobalConfig, StudyConfig, expand_tilde, parse_course_code,
    save_course_config,
};
use std::collections::HashMap;
use std::io::{self, Write};

pub fn run(global: &GlobalConfig, name: Option<&str>, raw: bool) -> io::Result<()> {
    let code = match name {
        Some(n) => n.to_string(),
        None => prompt_course_details()?,
    };

    let course_name = name
        .is_none()
        .then(|| prompt_course_name())
        .transpose()?
        .flatten();

    let courses_dir = expand_tilde(&global.courses_dir);

    let (faculty, dir_name) = if raw {
        ("undefined".to_string(), code.clone())
    } else {
        parse_course_code(&code)
    };

    let course_dir = courses_dir.join(&faculty).join(&dir_name);

    if course_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Course directory already exists: {}", course_dir.display()),
        ));
    }

    std::fs::create_dir_all(&course_dir)?;

    let config = CourseConfig {
        course: CourseInfo {
            code: code.clone(),
            name: course_name,
        },
        template_dir: None,
        exercise_types: HashMap::new(),
        study: StudyConfig::default(),
    };

    save_course_config(&course_dir, &config)?;

    println!("Initialized course '{}' at {}", code, course_dir.display());
    println!(
        "Edit {} to configure exercise types and study commands.",
        course_dir.join("course.toml").display()
    );

    Ok(())
}

fn prompt_course_details() -> io::Result<String> {
    print!("Course code: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let code = input.trim().to_string();

    if code.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Course code is required.",
        ));
    }

    Ok(code)
}

fn prompt_course_name() -> io::Result<Option<String>> {
    print!("Course name (leave empty for none): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let name = input.trim().to_string();

    Ok((!name.is_empty()).then_some(name))
}
