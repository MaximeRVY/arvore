use std::io;

use anyhow::Result;
use clap::CommandFactory;
use clap_complete::generate;

use crate::cli::{Cli, ShellType};

pub fn run(shell: &ShellType) -> Result<()> {
    let mut cmd = Cli::command();
    let shell_variant: clap_complete::Shell = shell.clone().into();
    generate(shell_variant, &mut cmd, "arvore", &mut io::stdout());
    Ok(())
}
