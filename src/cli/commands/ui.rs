use crate::config;
use crate::tui;
use anyhow::Result;

pub async fn ui(
    moldx_dir_override: Option<&std::path::Path>,
    bin_dir_override: Option<&std::path::Path>,
) -> Result<()> {
    let cfg = config::MoldxConfig::resolve(
        &std::env::current_dir()?,
        moldx_dir_override,
        bin_dir_override,
    )?;
    tui::run(cfg).await?;
    Ok(())
}
