use crate::config::{course_dirs, load_course_config, load_global_config};
use std::path::Path;

/// Scan `<courses_dir>/<faculty>/<course>/course.toml` to collect course codes and names.
pub fn list_courses() -> Vec<String> {
    let Ok(global) = load_global_config() else {
        return Vec::new();
    };

    course_dirs(&global)
        .filter_map(|dir| load_course_config(&dir).ok())
        .flat_map(|config| {
            std::iter::once(config.course.code).chain(config.course.name)
        })
        .collect()
}

/// Parse course.toml from a course directory and return exercise type keys.
pub fn list_exercise_types(course_dir: &Path) -> Vec<String> {
    load_course_config(course_dir)
        .map(|config| config.exercise_types.into_keys().collect())
        .unwrap_or_default()
}
