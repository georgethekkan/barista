use std::{env, path::PathBuf};

use anyhow::{Ok, Result};
use tracing::debug;
use walkdir::WalkDir;

pub async fn list_files<P: Into<PathBuf>>(path: P) -> Result<Vec<PathBuf>> {
    let path = path.into();
    let file_names = tokio::task::spawn_blocking(move || {
        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| entry.into_path())
            .collect::<Vec<_>>()
    })
    .await?;
    Ok(file_names)
}

pub fn join_paths(paths: &[PathBuf]) -> Result<String> {
    // Safely join paths using the correct OS separator (':' or ';')
    // Filter to existing directories and .jar files to avoid passing checksums/poms to classpath
    let valid_paths: Vec<_> = paths
        .iter()
        .filter(|p| {
            p.is_dir()
                || p.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("jar"))
                    .unwrap_or(false)
                || !p.exists() // Allow non-existent target directories that will be created
        })
        .collect();

    let classpath = env::join_paths(valid_paths)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    let classpath_str = classpath.to_string_lossy();
    debug!("Generated classpath: {}", classpath_str);

    Ok(classpath_str.to_string())
}
