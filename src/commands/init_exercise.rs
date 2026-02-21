use crate::config::{
    CourseConfig, CourseInfo, ExerciseType, GlobalConfig, expand_tilde, find_course_dir,
    find_course_root, load_course_config, load_state, save_state,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub fn run(
    global: &GlobalConfig,
    exercise_type: Option<&str>,
    custom_name: Option<&str>,
    course: Option<&str>,
) -> io::Result<()> {
    let course_dir = match course {
        Some(name) => find_course_dir(global, name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Course not found: {}", name),
            )
        })?,
        None => {
            let cwd = std::env::current_dir()?;
            find_course_root(&cwd).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "No course.toml found in current or parent directories. \
                     Use --course <name> or run from inside a course directory.",
                )
            })?
        }
    };

    let course_config = load_course_config(&course_dir)?;

    if course_config.exercise_types.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "No exercise types defined in course.toml. Add [exercise_types.<name>] sections.",
        ));
    }

    let (type_name, ex_type) = resolve_exercise_type(&course_config, exercise_type)?;

    let exercise_name = match custom_name {
        Some(name) => name.to_string(),
        None => next_name(&course_dir, &ex_type.naming_scheme)?,
    };

    let exercise_dir = course_dir.join(&exercise_name);
    if exercise_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "Exercise directory already exists: {}",
                exercise_dir.display()
            ),
        ));
    }

    if let Some(tpl) = resolve_template_dir(global, &course_config, &ex_type.template) {
        copy_dir_recursive(&tpl, &exercise_dir)?;
        println!(
            "Initialized {} exercise '{}' from template '{}'",
            type_name, exercise_name, ex_type.template
        );
    } else {
        fs::create_dir_all(&exercise_dir)?;
        println!(
            "Initialized {} exercise '{}' (no template found)",
            type_name, exercise_name
        );
    }

    generate_meta(
        &exercise_dir,
        &course_config.course,
        &exercise_name,
        &type_name,
        &ex_type.meta,
    )?;

    let mut state = load_state();
    state.last_course = Some(course_config.course.code.clone());
    state
        .last_exercises
        .insert(course_config.course.code.clone(), exercise_name.clone());
    save_state(&state)?;

    Ok(())
}

fn resolve_exercise_type<'a>(
    config: &'a CourseConfig,
    requested: Option<&str>,
) -> io::Result<(String, &'a ExerciseType)> {
    match requested {
        Some(name) => config
            .exercise_types
            .get(name)
            .map(|et| (name.to_string(), et))
            .ok_or_else(|| {
                let available: Vec<&str> = config.exercise_types.keys().map(|s| s.as_str()).collect();
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!(
                        "Unknown exercise type '{}'. Available: {}",
                        name,
                        available.join(", ")
                    ),
                )
            }),
        None => {
            if config.exercise_types.len() == 1 {
                let (name, et) = config.exercise_types.iter().next().unwrap();
                Ok((name.clone(), et))
            } else {
                let available: Vec<&str> = config.exercise_types.keys().map(|s| s.as_str()).collect();
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Multiple exercise types available. Use --type <type> to choose: {}",
                        available.join(", ")
                    ),
                ))
            }
        }
    }
}

fn next_name(course_dir: &Path, naming_scheme: &str) -> io::Result<String> {
    let (prefix, width) = parse_naming_scheme(naming_scheme);

    let max_num = fs::read_dir(course_dir)?
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let file_name = entry.file_name();
            let lossy = file_name.to_string_lossy();
            lossy
                .strip_prefix(&prefix)
                .and_then(|suffix| suffix.parse::<u32>().ok())
        })
        .max()
        .unwrap_or(0);

    Ok(format!("{}{:0>width$}", prefix, max_num + 1, width = width))
}

fn parse_naming_scheme(scheme: &str) -> (String, usize) {
    scheme.find('{').map_or(
        (scheme.to_string(), 2),
        |pos| {
            let width = scheme[pos..]
                .trim_start_matches("{:0")
                .trim_end_matches('}')
                .parse()
                .unwrap_or(2);
            (scheme[..pos].to_string(), width)
        },
    )
}

fn resolve_template_dir(
    global: &GlobalConfig,
    course: &CourseConfig,
    template_name: &str,
) -> Option<PathBuf> {
    let base = course
        .template_dir
        .as_deref()
        .unwrap_or(&global.default_template_dir);
    let path = expand_tilde(base).join(template_name);
    path.is_dir().then_some(path)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    fs::read_dir(src)?.try_for_each(|entry| {
        let entry = entry?;
        let dest = dst.join(entry.file_name());
        match entry.file_type()? {
            ft if ft.is_dir() => copy_dir_recursive(&entry.path(), &dest),
            _ => fs::copy(&entry.path(), &dest).map(|_| ()),
        }
    })
}

fn generate_meta(
    exercise_dir: &Path,
    course: &CourseInfo,
    exercise_name: &str,
    type_name: &str,
    custom_meta: &HashMap<String, toml::Value>,
) -> io::Result<()> {
    let mut meta = toml::map::Map::new();

    let mut course_table = toml::map::Map::new();
    course_table.insert("code".into(), toml::Value::String(course.code.clone()));
    if let Some(name) = &course.name {
        course_table.insert("name".into(), toml::Value::String(name.clone()));
    }
    meta.insert("course".into(), toml::Value::Table(course_table));

    let mut exercise_table = toml::map::Map::new();
    exercise_table.insert("name".into(), toml::Value::String(exercise_name.into()));
    exercise_table.insert("type".into(), toml::Value::String(type_name.into()));
    for (key, value) in custom_meta {
        exercise_table.insert(key.clone(), value.clone());
    }
    meta.insert("exercise".into(), toml::Value::Table(exercise_table));

    let contents = toml::to_string_pretty(&meta)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(exercise_dir.join("meta.toml"), contents)
}
