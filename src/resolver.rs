use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Ok, Result};
use ristretto_resolver::{ArtifactCoordinate, FileDestination, ResolutionRequest, Resolver};
use tokio::time::Instant;
use tracing::{debug, info, instrument};

use crate::io_util;
use crate::workspace::ExternalDependency;

#[instrument(name = "download_dependencies", skip_all)]
pub async fn download(deps: &[ExternalDependency], target_dir: &Path) -> Result<()> {
    inner_download(deps, target_dir).await
}

async fn inner_download(deps: &[ExternalDependency], target_dir: &Path) -> Result<()> {
    let downloaded = io_util::list_files(target_dir)?
        .into_iter()
        .map(|p| p.file_name().and_then(|n| n.to_str()).map(|s| s.to_string()))
        .flatten()
        .collect::<HashSet<_>>();

    let artifacts: Vec<ExternalDependency> = deps
        .iter()
        .filter(|dep| !downloaded.contains(&dep.file_name))
        .cloned()
        .collect();

    if artifacts.is_empty() {
        info!("All dependencies upto date");
        return Ok(());
    }

    let req_len = artifacts.len();

    let start = Instant::now();
    info!("Downloading {} dependencies", req_len);

    let mut join_handles = Vec::new();
    let target_path_buf = target_dir.to_path_buf();
    for art in artifacts {
        let coord = art.coordinate.clone();
        let path = art.local_jar_path.clone();
        let dest = target_path_buf.clone();
        let jh = tokio::spawn(downloaded_file(coord, path, dest));
        join_handles.push(jh);
    }

    let mut total_count = 0;
    for jh in join_handles {
        let count = jh.await??;
        total_count += count;
    }

    let tt = start.elapsed().as_secs();
    info!("Downloaded {req_len} dependencies, {total_count} transitive dependencies in {tt} seconds");

    Ok(())
}

async fn downloaded_file(coordinate: ArtifactCoordinate, path: PathBuf, target_dir: PathBuf) -> Result<usize> {
    let resolver = Resolver::builder().build()?;

    debug!("downloading file {:?}", path);
    let req = ResolutionRequest::new(coordinate);

    let resolution = resolver.resolve(&req).await?;
    let transitive_deps = resolution.transitive_dependencies();
    for dependency in &transitive_deps {
        debug!("{}: {:?}", dependency.coordinate, dependency.status);
    }

    let report = resolver
        .download(&resolution, &FileDestination::new(target_dir))
        .await?;

    debug!("downloaded {} artifacts", report.artifacts.len());
    Ok(transitive_deps.len())
}
