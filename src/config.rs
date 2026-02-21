use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io};

const COURSE_CONFIG_FILENAME: &str = "course.toml";
const STATE_FILENAME: &str = "state.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub courses_dir: String,
    pub default_template_dir: String,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            courses_dir: "~/courses".to_string(),
            default_template_dir: "~/.config/study/templates".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseConfig {
    pub course: CourseInfo,
    #[serde(default)]
    pub template_dir: Option<String>,
    #[serde(default)]
    pub exercise_types: HashMap<String, ExerciseType>,
    #[serde(default)]
    pub study: StudyConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseInfo {
    pub code: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExerciseType {
    pub template: String,
    pub naming_scheme: String,
    #[serde(default)]
    pub meta: HashMap<String, toml::Value>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StudyConfig {
    #[serde(default)]
    pub commands: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StudyState {
    #[serde(default)]
    pub last_course: Option<String>,
    #[serde(default)]
    pub last_exercises: HashMap<String, String>,
}

pub fn state_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("study")
        .join(STATE_FILENAME)
}

pub fn load_state() -> StudyState {
    let path = state_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|contents| toml::from_str(&contents).ok())
        .unwrap_or_default()
}

pub fn save_state(state: &StudyState) -> io::Result<()> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents =
        toml::to_string_pretty(state).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(&path, contents)
}

/// Find the most recently modified subdirectory in a course directory.
///
/// Skips hidden directories and `course.toml`.
pub fn find_latest_exercise(course_dir: &Path) -> Option<String> {
    fs::read_dir(course_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter(|e| {
            !e.file_name()
                .to_string_lossy()
                .starts_with('.')
        })
        .filter_map(|e| {
            e.metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| (e.file_name().to_string_lossy().into_owned(), t))
        })
        .max_by_key(|(_, t)| *t)
        .map(|(name, _)| name)
}

pub fn expand_tilde(path: &str) -> PathBuf {
    path.strip_prefix("~/")
        .and_then(|rest| dirs::home_dir().map(|home| home.join(rest)))
        .unwrap_or_else(|| PathBuf::from(path))
}

pub fn global_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("study")
        .join("config.toml")
}

pub fn load_global_config() -> io::Result<GlobalConfig> {
    let path = global_config_path();
    if !path.exists() {
        return Ok(GlobalConfig::default());
    }
    let contents = fs::read_to_string(&path)?;
    toml::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn save_global_config(config: &GlobalConfig) -> io::Result<()> {
    let path = global_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents =
        toml::to_string_pretty(config).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(&path, contents)
}

pub fn load_course_config(course_dir: &Path) -> io::Result<CourseConfig> {
    let path = course_dir.join(COURSE_CONFIG_FILENAME);
    let contents = fs::read_to_string(&path)?;
    toml::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn save_course_config(course_dir: &Path, config: &CourseConfig) -> io::Result<()> {
    let path = course_dir.join(COURSE_CONFIG_FILENAME);
    let contents =
        toml::to_string_pretty(config).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(&path, contents)
}

pub fn find_course_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| dir.join(COURSE_CONFIG_FILENAME).exists())
        .map(Path::to_path_buf)
}

/// Parse a course code into (faculty, course_code).
///
/// Rules:
/// 1. If input contains a hyphen: split on first `-`. Faculty = left, code = right.
/// 2. Else: faculty = longest leading alphabetical substring, code = rest.
/// 3. If no leading alpha: faculty = "undefined", code = full input.
pub fn parse_course_code(input: &str) -> (String, String) {
    if let Some(pos) = input.find('-') {
        let faculty = &input[..pos];
        let code = &input[pos + 1..];
        return (faculty.to_string(), code.to_string());
    }

    let alpha_len = input.chars().take_while(|c| c.is_ascii_alphabetic()).count();
    if alpha_len > 0 && alpha_len < input.len() {
        let faculty = &input[..alpha_len];
        let code = &input[alpha_len..];
        return (faculty.to_string(), code.to_string());
    }

    ("undefined".to_string(), input.to_string())
}

/// Resolve a course code to its expected directory path: `<courses_dir>/<faculty>/<course_code>/`
pub fn resolve_course_dir(global: &GlobalConfig, input: &str) -> PathBuf {
    let (faculty, code) = parse_course_code(input);
    expand_tilde(&global.courses_dir).join(faculty).join(code)
}

/// Iterate all course directories under `<courses_dir>/<faculty>/<course>/`.
pub fn course_dirs(global: &GlobalConfig) -> impl Iterator<Item = PathBuf> {
    let courses_dir = expand_tilde(&global.courses_dir);
    fs::read_dir(courses_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .flat_map(|faculty| {
            fs::read_dir(faculty.path())
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        })
        .map(|e| e.path())
}

/// Find the course directory for a given input (code or human-readable name).
///
/// First tries code-based path resolution, then falls back to scanning all
/// course.toml files for a matching `name`.
pub fn find_course_dir(global: &GlobalConfig, input: &str) -> Option<PathBuf> {
    let dir = resolve_course_dir(global, input);
    if dir.join(COURSE_CONFIG_FILENAME).exists() {
        return Some(dir);
    }

    course_dirs(global).find(|dir| {
        fs::read_to_string(dir.join(COURSE_CONFIG_FILENAME))
            .ok()
            .and_then(|contents| toml::from_str::<CourseConfig>(&contents).ok())
            .and_then(|config| config.course.name)
            .is_some_and(|name| name == input)
    })
}
