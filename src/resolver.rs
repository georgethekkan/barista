use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{Ok, Result};
use ristretto_resolver::{ArtifactCoordinate, FileDestination, ResolutionRequest, Resolver};
use tokio::time::Instant;
use tracing::{debug, info, instrument};
use walkdir::WalkDir;

use crate::cfg;
use crate::workspace::ResolvedDependency;

#[instrument(name = "download_test", skip_all)]
pub async fn download_test_dependencies(deps: &[ResolvedDependency]) -> Result<()> {
    inner_download(deps, cfg::TEST_DEPENDENCIES_FOLDER).await
}

#[instrument(name = "download_main", skip_all)]
pub async fn download(deps: &[ResolvedDependency]) -> Result<()> {
    inner_download(deps, cfg::DEPENDENCIES_FOLDER).await
}

async fn inner_download(deps: &[ResolvedDependency], target_folder: &str) -> Result<()> {
    let downloaded_files = list_download_files(target_folder)?;

    let artifacts: Vec<ResolvedDependency> = deps
        .iter()
        .filter_map(|dep| to_external(dep, &downloaded_files))
        .collect();

    if artifacts.is_empty() {
        info!("All dependencies upto date");
        return Ok(());
    }

    let req_len = artifacts.len();

    let start = Instant::now();
    info!("Downloading {} dependencies", req_len);

    let mut join_handles = Vec::new();
    for art in artifacts {
        if let ResolvedDependency::External {
            coordinate,
            local_jar_path,
            ..
        } = &art
        {
            let coord = coordinate.clone();
            let path = local_jar_path.clone();
            let jh = tokio::spawn(downloaded_file(coord, path, target_folder.to_string()));
            join_handles.push(jh);
        }
    }

    let mut total_count = 0;
    for jh in join_handles {
        let count = jh.await??;
        total_count += count;
    }

    let tt = start.elapsed().as_secs();
    info!("Downloaded {req_len} dependencies in {tt} seconds, {total_count} transitive dependencies");

    Ok(())
}

fn to_external(dep: &ResolvedDependency, downloaded: &HashSet<String>) -> Option<ResolvedDependency> {
    match dep {
        ResolvedDependency::External { file_name, .. } if !downloaded.contains(file_name) => Some(dep.clone()),
        _ => None,
    }
}

async fn downloaded_file(coordinate: ArtifactCoordinate, path: PathBuf, target_folder: String) -> Result<usize> {
    let resolver = Resolver::builder().build()?;

    debug!("downloading file {:?}", path);
    let req = ResolutionRequest::new(coordinate);

    let resolution = resolver.resolve(&req).await?;
    let transitive_deps = resolution.transitive_dependencies();
    for dependency in &transitive_deps {
        debug!("{}: {:?}", dependency.coordinate, dependency.status);
    }

    let report = resolver
        .download(&resolution, &FileDestination::new(target_folder))
        .await?;

    debug!("downloaded {} artifacts", report.artifacts.len());
    Ok(transitive_deps.len())
}

pub fn list_download_files(path: &str) -> Result<HashSet<String>> {
    let file_names = WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.file_name().to_os_string().into_string().ok())
        .collect();

    Ok(file_names)
}
