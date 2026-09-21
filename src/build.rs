use anyhow::{Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use tokio::time::Instant;
use tracing::{info, instrument};

use crate::{
    cfg,
    io_util::{join_paths, list_files},
    manifest::Dependency,
};

#[instrument(name = "build_main", skip_all)]
pub async fn build(_dependencies: &[Dependency], _base_dir: &str) -> Result<()> {
    inner_build(
        cfg::SRC_FOLDER,
        cfg::CLASSES_FOLDER,
        &[cfg::DEPENDENCIES_FOLDER],
        &[],
    )
    .await
}

#[instrument(name = "build_test", skip_all)]
pub async fn build_test(_dependencies: &[Dependency], _base_dir: &str) -> Result<()> {
    inner_build(
        cfg::TEST_SRC_FOLDER,
        cfg::TEST_CLASSES_FOLDER,
        &[cfg::TEST_DEPENDENCIES_FOLDER, cfg::DEPENDENCIES_FOLDER],
        &[cfg::CLASSES_FOLDER],
    )
    .await
}

async fn inner_build(
    src_folder: &str,
    cls_folder: &str,
    jar_dirs: &[&str],
    class_dirs: &[&str],
) -> Result<()> {
    let start = Instant::now();

    let sources: Vec<PathBuf> = list_files(src_folder)
        .await?
        .into_iter()
        .filter(|p| p.extension().is_some_and(|ext| ext == "java"))
        .collect();

    let sources = sources_to_compile(sources, src_folder, cls_folder).await?;

    if sources.is_empty() {
        info!("Incremental build: no sources to compile.");
        return Ok(());
    }

    let mut jar_paths = Vec::new();
    jar_paths.push(PathBuf::from(cls_folder));
    for dir in class_dirs {
        jar_paths.push(PathBuf::from(dir));
    }
    for dir in jar_dirs {
        jar_paths.extend(list_files(dir).await?);
    }
    let classpath_str = join_paths(&jar_paths)?;

    // 3. Build and execute the javac system process command
    let output = Command::new("javac")
        .arg("-cp")
        .arg(classpath_str.as_str()) // Passes the formatted classpath string
        .arg("-d")
        .arg(cls_folder) // Output directory for compiled .class files
        .args(&sources) // Your target source file(s)
        .output()?; // Executes the process and awaits termination

    // 4. Handle stderr/stdout outputs from the compiler
    if output.status.success() {
        info!(
            "Compiled Java {} files in {}ms",
            sources.len(),
            start.elapsed().as_millis()
        );
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Java compilation failed:\n{}", stderr);
    }
}

async fn sources_to_compile(
    sources: Vec<PathBuf>,
    src_folder: &str,
    cls_folder: &str,
) -> Result<Vec<PathBuf>> {
    let mut to_compile = Vec::new();
    let src_dir = Path::new(src_folder);
    let class_dir = Path::new(cls_folder);

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
