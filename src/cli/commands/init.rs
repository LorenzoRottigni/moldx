use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub async fn init(
    moldx_dir_override: Option<&Path>,
    bin_dir_override: Option<&Path>,
) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let moldx_dir: PathBuf = if let Some(p) = moldx_dir_override {
        p.to_path_buf()
    } else {
        cwd.join(".moldx")
    };

    // Determine bin dir (either override or <moldx_dir>/bin)
    let bin_dir: PathBuf = if let Some(b) = bin_dir_override {
        b.to_path_buf()
    } else {
        moldx_dir.join("bin")
    };

    // Create directories
    if !moldx_dir.exists() {
        fs::create_dir_all(&moldx_dir)?;
        println!("Created {}", moldx_dir.display());
    } else {
        println!("Directory already exists: {}", moldx_dir.display());
    }

    if !bin_dir.exists() {
        fs::create_dir_all(&bin_dir)?;
        println!("Created {}", bin_dir.display());
    } else {
        println!("Directory already exists: {}", bin_dir.display());
    }

    // Write probe.sh if missing
    let probe_path = moldx_dir.join("probe.sh");
    if probe_path.exists() {
        println!("probe.sh already exists: {}", probe_path.display());
    } else {
        let mut f = fs::File::create(&probe_path)?;
        let content = "#!/usr/bin/env bash\nTARGET=\"$1\"\n[ -f \"$TARGET/Dockerfile\" ]   && echo \"docker\"\n[ -f \"$TARGET/package.json\" ] && echo \"node\"\n[ -f \"$TARGET/Cargo.toml\" ]   && echo \"rust\"\n";
        f.write_all(content.as_bytes())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&probe_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&probe_path, perms)?;
        }
        println!("Wrote probe script: {}", probe_path.display());
    }

    // Write .moldx/README.md
    let readme_path = moldx_dir.join("README.md");
    if readme_path.exists() {
        println!("README.md already exists: {}", readme_path.display());
    } else {
        let content = "# .moldx\n\nThis directory contains project-local configuration for moldx.\n\n- `probe.sh`: detects strategies for a given module path.\n- `bin/`: command scripts (strategy-specific and agnostic).\n\nSee the top-level moldx README for full documentation.\n";
        fs::write(&readme_path, content)?;
        println!("Wrote {}", readme_path.display());
    }

    // Write .moldx/bin/README.md
    let bin_readme = bin_dir.join("README.md");
    if bin_readme.exists() {
        println!("bin README already exists: {}", bin_readme.display());
    } else {
        let content = "# .moldx/bin\n\nPlace command scripts here. Two layouts are supported:\n\n- Strategy-agnostic: `.moldx/bin/<command>.sh`\n- Strategy-specific: `.moldx/bin/<command>/<strategy>.sh`\n\nEach script receives the absolute module path as `$1` and should forward its exit code.\n";
        fs::write(&bin_readme, content)?;
        println!("Wrote {}", bin_readme.display());
    }

    Ok(())
}
