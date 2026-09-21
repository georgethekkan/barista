use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{Ok, Result};
use ristretto_resolver::{ArtifactCoordinate, FileDestination, ResolutionRequest, Resolver};
use tokio::time::Instant;
use tracing::{debug, info, instrument};
use walkdir::WalkDir;

use crate::cfg;
use crate::manifest::Dependency;

#[instrument(name = "download_test", skip_all)]
pub async fn download_test_dependencies(deps: &[Dependency]) -> Result<()> {
    inner_download(deps, cfg::TEST_DEPENDENCIES_FOLDER).await
}

#[instrument(name = "download_main", skip_all)]
pub async fn download(deps: &[Dependency]) -> Result<()> {
    inner_download(deps, cfg::DEPENDENCIES_FOLDER).await
}

async fn inner_download(deps: &[Dependency], target_folder: &str) -> Result<()> {
    let downloaded_files = list_download_files(target_folder).await?;

    let artifacts = deps
        .iter()
        .filter(|it| !downloaded_files.contains(&it.file))
        .collect::<Vec<_>>();

    if artifacts.is_empty() {
        info!("All dependencies upto date");
        return Ok(());
    }

    let req_len = artifacts.len();

    let start = Instant::now();
    info!("Downloading {} dependencies", req_len);

    let mut join_handles = Vec::new();
    for art in artifacts {
        let coord = art.coordinate.clone();
        let path = art.path.clone();
        let jh = tokio::spawn(downloaded_file(coord, path, target_folder.to_string()));
        join_handles.push(jh);
    }

    for jh in join_handles {
        jh.await??;
    }

    let tt = start.elapsed().as_secs();
    info!("Downloaded {} dependencies in {} seconds", req_len, tt);

    Ok(())
}

async fn downloaded_file(
    coordinate: ArtifactCoordinate,
    path: PathBuf,
    target_folder: String,
) -> Result<()> {
    let resolver = Resolver::builder().build()?;

    debug!("downloading file {:?}", path);
    let req = ResolutionRequest::new(coordinate);

    let resolution = resolver.resolve(&req).await?;
    for dependency in resolution.transitive_dependencies() {
        debug!("{}: {:?}", dependency.coordinate, dependency.status);
    }
    let report = resolver
        .download(&resolution, &FileDestination::new(target_folder))
        .await?;

    debug!("downloaded {} artifacts", report.artifacts.len());
    Ok(())
}

pub async fn list_download_files(path: &str) -> Result<HashSet<String>> {
    let path = path.to_string();
    let file_names = tokio::task::spawn_blocking(move || {
        WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| e.file_name().to_os_string().into_string().ok())
            .collect::<HashSet<_>>()
    })
    .await?;

    Ok(file_names)
}
