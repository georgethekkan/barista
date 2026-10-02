use std::{fs, path::Path};

use anyhow::{Context, Ok, Result, anyhow};

use crate::workspace::Workspace;

pub mod args;
pub mod build;
pub mod cfg;
pub mod io_util;
pub mod manifest;
pub mod resolver;
pub mod run;
mod test_runner;
pub mod tracing;
pub mod workspace;

pub fn clean() -> Result<()> {
    if !Path::new(cfg::TARGET_FOLDER).exists() {
        return Ok(());
    }

    fs::remove_dir_all(cfg::TARGET_FOLDER).context("error deleting target folder")
}

pub fn load_workspace(path: &str) -> Result<Workspace> {
    Workspace::from_root_dir(path, cfg::CONFIG_FILE)
}

pub async fn build() -> Result<()> {
    let ws = inner_build().await?;
    test_runner::run(&ws)?;

    Ok(())
}

pub async fn run(bin_args: &[String]) -> Result<()> {
    let ws = inner_build().await?;

    let configured_main = ws.root_manifest.package.main_class.as_deref();
    let (main_class, app_args) = match configured_main {
        Some(main) => (main, bin_args),
        None => {
            let (first, rest) = bin_args
                .split_first()
                .ok_or_else(|| anyhow!("No main class provided and no default configured in barista.toml"))?;
            (first.as_str(), rest)
        }
    };

    run::run(main_class, app_args, &ws)?;
    Ok(())
}

async fn inner_build() -> Result<Workspace> {
    let ws = load_workspace(cfg::ROOT_DIR)?;

    let main_deps = ws.all_external_dependencies();
    let dev_deps = ws.all_external_dev_dependencies();

    tokio::try_join!(
        resolver::download(&main_deps),
        resolver::download_test_dependencies(&dev_deps),
    )?;

    // Trigger compilation steps
    build::build(&ws)?;
    build::build_test(&ws)?;

    Ok(ws)
}
