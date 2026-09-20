use std::{env, path::PathBuf};

use anyhow::{Ok, Result};
use tracing::debug;
use walkdir::WalkDir;

pub async fn list_files(path: &str) -> Result<Vec<PathBuf>> {
    let path = PathBuf::from(path);
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
    //Safely join paths using the correct OS separator (':' or ';')
    let classpath = env::join_paths(paths)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    let classpath_str = classpath.to_string_lossy();
    debug!("Generated classpath: {}", classpath_str);

    Ok(classpath_str.to_string())
}
