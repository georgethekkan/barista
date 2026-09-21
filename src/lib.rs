use std::{fs, path::Path};

use anyhow::{Context, Ok, Result, anyhow};

use crate::manifest::{Dependency, Manifest};

pub mod args;
pub mod build;
pub mod cfg;
pub mod io_util;
pub mod manifest;
pub mod resolver;
pub mod run;
mod test_runner;
pub mod tracing;

pub fn clean() -> Result<()> {
    if !Path::new(cfg::TARGET_FOLDER).exists() {
        return Ok(());
    }

    fs::remove_dir_all(cfg::TARGET_FOLDER).context("error deleting target folder")
}

pub async fn build(args: Vec<String>) -> Result<()> {
    let (_mf, _deps) = inner_build(&args).await?;
    test_runner::run().await?;

    Ok(())
}

pub async fn run(args: &[String]) -> Result<()> {
    let (mf, deps) = inner_build(args).await?;

    let main_class = args
        .first()
        .map(String::as_str)
        .or(mf.package.main_class.as_deref())
        .ok_or_else(|| {
            anyhow!("No main class provided and no default configured in barista.toml")
        })?;

    run::run(deps, main_class).await?;
    Ok(())
}

async fn inner_build(_args: &[String]) -> Result<(Manifest, Vec<Dependency>)> {
    let mf = manifest::load()?;

    let deps = mf.dependencies();
    let dev_deps = mf.dev_dependencies();

    tokio::try_join!(
        resolver::download(&deps),
        resolver::download_test_dependencies(&dev_deps),
    )?;

    build::build(&deps, cfg::DEPENDENCIES_FOLDER).await?;
    build::build_test(&dev_deps, cfg::TEST_DEPENDENCIES_FOLDER).await?;

    Ok((mf, deps))
}
