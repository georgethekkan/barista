use std::{collections::HashSet, path::Path};

use anyhow::{Ok, Result};
use ristretto_resolver::{ArtifactCoordinate, FileDestination, ResolutionRequest, Resolver};
use tokio::time::Instant;
use tracing::{debug, info};
use walkdir::WalkDir;

use crate::cfg;
use crate::deps::Dependency;

pub async fn download(deps: &Vec<Dependency>) -> Result<()> {
    let downloaded_files = list_download_files(cfg::DEPENDENCIES_FOLDER).await?;

    let artifacts = deps
        .iter()
        .filter(|it| !downloaded_files.contains(&it.file))
        .collect::<Vec<_>>();

    if artifacts.is_empty() {
        info!("All dependencies are already downloaded");
        return Ok(());
    }

    let req_len = artifacts.len();

    let start = Instant::now();
    info!("Downloading {} dependencies", req_len);

    let resolver = Resolver::builder().build()?;

    for art in artifacts {
        println!("{:?}", art.path);
        let req = ResolutionRequest::new(art.coordinate.clone());

        let resolution = resolver.resolve(&req).await?;
        for dependency in resolution.transitive_dependencies() {
            debug!("{}: {:?}", dependency.coordinate, dependency.status);
        }
        let report = resolver
            .download(&resolution, &FileDestination::new(cfg::DEPENDENCIES_FOLDER))
            .await?;
        debug!("downloaded {} artifacts", report.artifacts.len());
    }

    let tt = start.elapsed().as_secs();
    info!("Downloaded {} dependencies in {} seconds", req_len, tt);

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
