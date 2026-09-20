use std::fs;

use anyhow::{Context, Ok, Result};

use crate::deps::Dependency;

pub mod args;
pub mod build;
pub mod cfg;
pub mod deps;
pub mod dm;
pub mod manifest;
pub mod run;
pub mod tracing;

pub fn clean() -> Result<()> {
    fs::remove_dir_all(cfg::TARGET_FOLDER).context("error deleting target folder")
}

pub async fn build(args: Vec<String>) -> Result<()> {
    let _deps = inner_build(&args).await?;

    Ok(())
}

pub async fn run(args: Vec<String>) -> Result<()> {
    let deps = inner_build(&args).await?;
    run::run(deps, args.get(0).unwrap())?;
    Ok(())
}

async fn inner_build(_args: &[String]) -> Result<Vec<Dependency>> {
    let mf = manifest::load()?;

    let deps = deps::to_dependencies(mf.dependency_configs())?;
    dm::download(&deps).await?;
    build::build(&deps, cfg::DEPENDENCIES_FOLDER)?;

    Ok(deps)
}
