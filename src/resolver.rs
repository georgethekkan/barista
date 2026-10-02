use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Ok, Result};
use ristretto_resolver::{ArtifactCoordinate, FileDestination, ResolutionRequest, Resolver};
use tokio::time::Instant;
use tracing::{debug, info, instrument};
use walkdir::WalkDir;

use crate::workspace::ResolvedDependency;

#[instrument(name = "download_dependencies", skip_all)]
pub async fn download(deps: &[ResolvedDependency], target_dir: &Path) -> Result<()> {
    inner_download(deps, target_dir).await
}

async fn inner_download(deps: &[ResolvedDependency], target_dir: &Path) -> Result<()> {
    let downloaded_files = list_download_files(target_dir)?;

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
    let target_path_buf = target_dir.to_path_buf();
    for art in artifacts {
        if let ResolvedDependency::External {
            coordinate,
            local_jar_path,
            ..
        } = &art
        {
            let coord = coordinate.clone();
            let path = local_jar_path.clone();
            let dest = target_path_buf.clone();
            let jh = tokio::spawn(downloaded_file(coord, path, dest));
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

pub fn list_download_files<P: AsRef<Path>>(path: P) -> Result<HashSet<String>> {
    let file_names = WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.file_name().to_os_string().into_string().ok())
        .collect();

    Ok(file_names)
}
