use crate::cli::Args;
use clap::CommandFactory;
use clap_complete::generate;
use std::io;

pub fn run(shell: clap_complete::Shell) -> io::Result<()> {
    let mut cmd = Args::command();
    generate(shell, &mut cmd, "study", &mut io::stdout());
    Ok(())
}
