use anyhow::Result;
use std::fs;
use std::fs::File;
use std::path::{PathBuf};

use crate::client::MoldXClient;

pub async fn init(
    client: &MoldXClient
) -> Result<()> {
    let moldx_dir: PathBuf = client.config.moldx_dir.clone();
    let strategies_dir: PathBuf = client.config.strategies_dir.clone();
    let bin_dir_name: String = client.config.bin_dir_name.clone();
    let template_dir_name: String = client.config.bin_dir_name.clone();

    if !strategies_dir.exists() {
        fs::create_dir_all(&strategies_dir)?;
        println!("Created {}", strategies_dir.display());
    } else {
        println!("Directory already exists: {}", strategies_dir.display());
    }

    
    let default_strategy_dir = strategies_dir.join("default");
    [
        default_strategy_dir.join(&bin_dir_name).join(".keep"),
        default_strategy_dir.join(&template_dir_name).join(".keep"),
    ]
        .iter()
        .try_for_each(|path| {
            fs::create_dir_all(path.parent().unwrap())?;
            File::create(path)?;
            Ok::<(), std::io::Error>(())
        })?;

    // Write .moldx/README.md
    let readme_path = moldx_dir.join("README.md");
    if readme_path.exists() {
        println!("README.md already exists: {}", readme_path.display());
    } else {
        let content = "# .moldx";
        fs::write(&readme_path, content)?;
        println!("Wrote {}", readme_path.display());
    }

    Ok(())
}
