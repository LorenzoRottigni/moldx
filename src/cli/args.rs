use anyhow::Result;

pub trait FromCommandArgs: Sized {
    fn from_command_args(args: Vec<String>) -> Result<Self>;
}
