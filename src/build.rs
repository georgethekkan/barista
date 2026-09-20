use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use tokio::time::Instant;
use tracing::{error, info};

use crate::{
    cfg,
    deps::Dependency,
    io_util::{join_paths, list_files},
};

pub async fn build(_dependencies: &[Dependency], _base_dir: &str) -> Result<()> {
    let start = Instant::now();

    let sources: Vec<PathBuf> = list_files(cfg::SRC_FOLDER)
        .await?
        .into_iter()
        .filter(|p| p.extension().is_some_and(|ext| ext == "java"))
        .collect();

    let sources = sources_to_compile(sources).await?;

    if sources.is_empty() {
        info!("Incremental build: no sources to compile.");
        return Ok(());
    }

    let jar_paths = list_files(cfg::DEPENDENCIES_FOLDER).await?;
    let classpath_str = join_paths(&jar_paths)?;

    // 3. Build and execute the javac system process command
    let output = Command::new("javac")
        .arg("-cp")
        .arg(&*classpath_str) // Passes the formatted classpath string
        .arg("-d")
        .arg(cfg::CLASSES_FOLDER) // Output directory for compiled .class files
        .args(&sources) // Your target source file(s)
        .output()?; // Executes the process and awaits termination

    // 4. Handle stderr/stdout outputs from the compiler
    if output.status.success() {
        info!(
            "Compiled Java {} files in {}ms",
            sources.len(),
            start.elapsed().as_millis()
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Java compilation failed:\n{}", stderr);
    }

    Ok(())
}

async fn sources_to_compile(sources: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    let mut sources_to_compile = Vec::new();
    for source in sources {
        let class_str = source
            .clone()
            .into_string()
            .unwrap()
            .replace(cfg::SRC_FOLDER, cfg::CLASSES_FOLDER)
            .replace(".java", ".class");
        let class_file = PathBuf::from(class_str);

        if !class_file.exists()
            || source.metadata().unwrap().modified().unwrap()
                > class_file.metadata().unwrap().modified().unwrap()
        {
            sources_to_compile.push(source);
        }
    }
    Ok(sources_to_compile)
}
