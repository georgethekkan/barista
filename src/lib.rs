use std::fs;

use anyhow::{Context, Ok, Result, anyhow};

use crate::{deps::Dependency, manifest::Manifest};

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
    let (_mf, _deps) = inner_build(&args).await?;

    Ok(())
}

pub async fn run(args: &[String]) -> Result<()> {
    let (mf, deps) = inner_build(&args).await?;

    let main_class = args
        .first()
        .map(String::as_str)
        .or(mf.package.main_class.as_deref())
        .ok_or_else(|| anyhow!("No main class provided and no default configured in jb.toml"))?;

    run::run(deps, main_class)?;
    Ok(())
}

async fn inner_build(_args: &[String]) -> Result<(Manifest, Vec<Dependency>)> {
    let mf = manifest::load()?;

    let deps = deps::to_dependencies(mf.dependency_configs())?;
    dm::download(&deps).await?;
    build::build(&deps, cfg::DEPENDENCIES_FOLDER)?;

    Ok((mf, deps))
}
