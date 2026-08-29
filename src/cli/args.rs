use anyhow::Result;
use clap::Parser;

pub trait FromCommandArgs: Sized {
    fn from_command_args(args: Vec<String>) -> Result<Self>;
}

#[derive(Parser)]
pub struct CommandArgs {
    #[arg(required = true)]
    pub args: Vec<String>,
}
