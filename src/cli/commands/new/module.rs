use crate::client::MoldXClient;
use anyhow::{Result, bail};
use std::{fs, path::PathBuf};

pub fn new_module(args: Vec<String>) -> Result<()> {
    let target_index = match args.len() {
        2 => 1,
        3 => 2,
        4 => 3,
        _ => bail!("Usage: moldx new module [strategy] [template] <module-path>"),
    };
    let module_path = PathBuf::from(&args[target_index]);
    if module_path.exists() {
        bail!("Module path already exists: {}", module_path.display());
    }
    fs::create_dir_all(&module_path)?;
    println!("Created module at {}", module_path.display());
    Ok(())
}