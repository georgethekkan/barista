use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use tokio::time::Instant;
use tracing::{error, info};

use crate::{
    cfg,
    io_util::{join_paths, list_files},
    manifest::Dependency,
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

    let mut jar_paths = list_files(cfg::DEPENDENCIES_FOLDER).await?;
    jar_paths.insert(0, PathBuf::from(cfg::CLASSES_FOLDER));
    let classpath_str = join_paths(&jar_paths)?;

    // 3. Build and execute the javac system process command
    let output = Command::new("javac")
        .arg("-cp")
        .arg(classpath_str.as_str()) // Passes the formatted classpath string
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
    let mut to_compile = Vec::new();
    let src_dir = Path::new(cfg::SRC_FOLDER);
    let class_dir = Path::new(cfg::CLASSES_FOLDER);

    let now = SystemTime::now();

    for source in sources {
        let relative = source.strip_prefix(src_dir).unwrap_or(&source);
        let class_file = class_dir.join(relative).with_extension("class");

        let needs_recompile = match (source.metadata(), class_file.metadata()) {
            (Ok(sm), Ok(cm)) => {
                sm.modified().unwrap_or(now) > cm.modified().unwrap_or(SystemTime::UNIX_EPOCH)
            }
            _ => true,
        };

        if needs_recompile {
            to_compile.push(source);
        }
    }
    Ok(to_compile)
}
