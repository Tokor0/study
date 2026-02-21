mod cli;
mod commands;
mod completions;
mod config;

use clap::Parser;
use cli::{Args, Command, InitTarget};
use config::load_global_config;

fn main() {
    let args = Args::parse();
    let global = match load_global_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading global config: {}", e);
            std::process::exit(1);
        }
    };

    let result = match args.command {
        Some(Command::Init { target }) => match target {
            InitTarget::Course { name, raw } => {
                commands::init_course::run(&global, name.as_deref(), raw)
            }
            InitTarget::Exercise {
                r#type,
                name,
                course,
            } => {
                commands::init_exercise::run(
                    &global,
                    r#type.as_deref(),
                    name.as_deref(),
                    course.as_deref(),
                )
            }
        },
        Some(Command::Study { course, exercise }) => {
            commands::study::run(&global, Some(&course), exercise.as_deref())
        }
        Some(Command::Completions { shell }) => commands::completions::run(shell),
        None => {
            commands::study::run(&global, args.course.as_deref(), args.exercise.as_deref())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
