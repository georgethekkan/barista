use std::{fs, path::Path};

use anyhow::{Context, Ok, Result, anyhow};

use crate::workspace::Workspace;

pub mod args;
pub mod build;
pub mod io_util;
pub mod manifest;
pub mod resolver;
pub mod run;
mod test_runner;
pub mod tracing;
pub mod workspace;

pub fn clean() -> Result<()> {
    let ws = load_workspace(".")?;
    let target = ws.target_dir();
    if !target.exists() {
        return Ok(());
    }

    fs::remove_dir_all(&target).context("error deleting target folder")
}

pub fn load_workspace<P: AsRef<Path>>(path: P) -> Result<Workspace> {
    Workspace::from_root_dir(path, manifest::MANIFEST_FILE)
}

pub async fn build() -> Result<()> {
    let ws = inner_build().await?;
    test_runner::run(&ws)?;

    Ok(())
}

pub async fn run(bin_args: &[String]) -> Result<()> {
    let ws = inner_build().await?;

    let mail_class = ws.root_manifest.package.main_class.as_deref();
    let (main_class, app_args) = match mail_class {
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
    let ws = load_workspace(".")?;

    let main_deps = ws.all_external_dependencies();
    let dev_deps = ws.all_external_dev_dependencies();
    let deps_dir = ws.dependencies_dir();
    let test_deps_dir = ws.test_dependencies_dir();

    tokio::try_join!(
        resolver::download(&main_deps, &deps_dir),
        resolver::download(&dev_deps, &test_deps_dir),
    )?;

    build::build(&ws)?;
    build::build_test(&ws)?;

    Ok(ws)
}
