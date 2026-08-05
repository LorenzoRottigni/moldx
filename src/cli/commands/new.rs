use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub async fn new(
    args: Vec<String>,
    moldx_dir_override: Option<&Path>,
    bin_dir_override: Option<&Path>,
) -> Result<()> {
    // parse args: either [command] or [strategy, command]
    if args.is_empty() {
        return Err(anyhow::anyhow!("Usage: moldx new [strategy] <command>"));
    }

    let (maybe_strategy, command) = if args.len() == 1 {
        (None, args[0].clone())
    } else if args.len() == 2 {
        (Some(args[0].clone()), args[1].clone())
    } else {
        return Err(anyhow::anyhow!("Usage: moldx new [strategy] <command>"));
    };

    let cwd = std::env::current_dir()?;
    let moldx_dir: PathBuf = if let Some(p) = moldx_dir_override {
        p.to_path_buf()
    } else {
        cwd.join(".moldx")
    };

    let bin_dir: PathBuf = if let Some(b) = bin_dir_override {
        b.to_path_buf()
    } else {
        moldx_dir.join("bin")
    };

    // Require that the .moldx directory (or the overridden path) already exists.
    if moldx_dir_override.is_some() {
        if !moldx_dir.exists() {
            return Err(anyhow::anyhow!(
                "MOLDX_DIR override does not exist: {}",
                moldx_dir.display()
            ));
        }
    } else if !moldx_dir.exists() {
        return Err(anyhow::anyhow!(
            "No .moldx directory found in the current directory. Run `moldx init` first."
        ));
    }

    // If `--bin-dir` was explicitly provided, require it exists. Otherwise create
    // a `bin/` directory inside the existing `.moldx/` when needed.
    if bin_dir_override.is_some() {
        if !bin_dir.exists() {
            return Err(anyhow::anyhow!(
                "MOLDX_BIN_DIR override does not exist: {}",
                bin_dir.display()
            ));
        }
    } else {
        if !bin_dir.exists() {
            fs::create_dir_all(&bin_dir)?;
            println!("Created {}", bin_dir.display());
        }
    }

    // create command dir
    let command_dir = bin_dir.join(&command);
    if !command_dir.exists() {
        fs::create_dir_all(&command_dir)?;
        println!("Created {}", command_dir.display());
    }

    // strategy-specific script
    if let Some(strategy) = maybe_strategy {
        let script_path = command_dir.join(format!("{}.sh", strategy));
        if script_path.exists() {
            println!("File already exists: {}", script_path.display());
        } else {
            let mut f = fs::File::create(&script_path)?;
            let content = format!(
                "#!/usr/bin/env bash\nset -euo pipefail\nMODULE_PATH=\"$1\"\necho \"Running {} for {}\"\n",
                strategy, command
            );
            f.write_all(content.as_bytes())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&script_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&script_path, perms)?;
            }
            println!("Wrote {}", script_path.display());
        }
    }

    // strategy-agnostic script
    let agnostic_path = bin_dir.join(format!("{}.sh", command));
    if agnostic_path.exists() {
        println!("File already exists: {}", agnostic_path.display());
    } else {
        let mut f = fs::File::create(&agnostic_path)?;
        let content = format!(
            "#!/usr/bin/env bash\nset -euo pipefail\nMODULE_PATH=\"$1\"\necho \"Running {}\"\n",
            command
        );
        f.write_all(content.as_bytes())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&agnostic_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&agnostic_path, perms)?;
        }
        println!("Wrote {}", agnostic_path.display());
    }

    Ok(())
}
