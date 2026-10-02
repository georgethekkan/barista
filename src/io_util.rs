use std::{
    env,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use std::io::Error as StdError;

use anyhow::{Ok, Result};
use tracing::debug;
use walkdir::WalkDir;

pub fn list_files<P: AsRef<Path>>(path: P) -> Result<impl Iterator<Item = PathBuf>> {
    let files = WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path());

    Ok(files)
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

    let classpath = env::join_paths(valid_paths).map_err(|e| StdError::new(ErrorKind::InvalidInput, e))?;

    let classpath_str = classpath.to_string_lossy();
    debug!("Generated classpath: {}", classpath_str);

    Ok(classpath_str.to_string())
}

pub fn build_classpath(class_dirs: &[&Path], jar_dirs: &[&Path]) -> Result<String> {
    let mut paths = Vec::new();
    for dir in class_dirs {
        paths.push(dir.to_path_buf());
    }
    for dir in jar_dirs {
        paths.extend(list_files(dir)?);
    }
    join_paths(&paths)
}
