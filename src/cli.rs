use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "study", about = "Course management CLI")]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Course name (shorthand for `study <course>`)
    pub course: Option<String>,

    /// Exercise name (defaults to last accessed)
    pub exercise: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a course or exercise
    Init {
        #[command(subcommand)]
        target: InitTarget,
    },
    /// Start a study session for a course
    Study {
        /// Course name
        course: String,
        /// Exercise name (defaults to last accessed)
        exercise: Option<String>,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

#[derive(Subcommand)]
pub enum InitTarget {
    /// Initialize a new course directory
    Course {
        /// Course code (e.g. MS-C2286, PHYS101); prompted if omitted
        name: Option<String>,
        /// Skip faculty parsing; place under undefined/<code>
        #[arg(short, long)]
        raw: bool,
    },
    /// Initialize a new exercise in the current course
    Exercise {
        /// Exercise type (from course config)
        #[arg(short, long)]
        r#type: Option<String>,
        /// Custom exercise directory name (overrides auto-naming)
        #[arg(short, long)]
        name: Option<String>,
        /// Course name (instead of detecting from current directory)
        #[arg(short, long)]
        course: Option<String>,
    },
}
