use anyhow::{Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use tokio::time::Instant;
use tracing::{info, instrument};

use crate::cfg;
use crate::io_util::{join_paths, list_files};
use crate::workspace::Workspace;

#[instrument(name = "build_main", skip_all)]
pub async fn build(ws: &Workspace) -> Result<()> {
    inner_build(
        ws,
        cfg::SRC_FOLDER,
        cfg::CLASSES_FOLDER,
        &[cfg::DEPENDENCIES_FOLDER],
        &[],
    )
    .await
}

#[instrument(name = "build_test", skip_all)]
pub async fn build_test(ws: &Workspace) -> Result<()> {
    inner_build(
        ws,
        cfg::TEST_SRC_FOLDER,
        cfg::TEST_CLASSES_FOLDER,
        &[cfg::TEST_DEPENDENCIES_FOLDER, cfg::DEPENDENCIES_FOLDER],
        &[cfg::CLASSES_FOLDER],
    )
    .await
}

async fn inner_build(
    ws: &Workspace,
    src_folder: &str,
    cls_folder: &str,
    jar_dirs: &[&str],
    class_dirs: &[&str],
) -> Result<()> {
    if ws.members.is_empty() {
        return compile_target(
            Path::new(src_folder),
            Path::new(cls_folder),
            jar_dirs,
            class_dirs,
        )
        .await;
    }

    // Resolve compile order and execute sequentially
    let compile_order = ws.resolve_compile_order();
    for member in compile_order {
        let member_src = member.path.join(src_folder);
        let member_cls = Path::new(cls_folder);
        info!(
            "Compiling member: {}, src_folder: {}, target_folder: {}",
            member.name,
            member_src.display(),
            member_cls.display(),
        );
        compile_target(&member_src, member_cls, jar_dirs, class_dirs).await?;
    }

    Ok(())
}

#[instrument(skip_all)]
async fn compile_target(
    src_dir: &Path,
    cls_dir: &Path,
    jar_dirs: &[&str],
    class_dirs: &[&str],
) -> Result<()> {
    let start = Instant::now();

    if !src_dir.exists() {
        return Ok(());
    }

    let sources: Vec<PathBuf> = list_files(src_dir)
        .await?
        .into_iter()
        .filter(|p| p.extension().is_some_and(|ext| ext == "java"))
        .collect();

    let sources = sources_to_compile(sources, src_dir, cls_dir).await?;

    if sources.is_empty() {
        info!("Incremental build: no sources to compile in {}", src_dir.display());
        return Ok(());
    }

    // Ensure output classes directory exists before compiling
    fs::create_dir_all(cls_dir)?;

    let mut jar_paths = Vec::new();
    jar_paths.push(cls_dir.to_path_buf());
    for dir in class_dirs {
        jar_paths.push(PathBuf::from(dir));
    }
    for dir in jar_dirs {
        jar_paths.extend(list_files(dir).await?);
    }
    let classpath_str = join_paths(&jar_paths)?;

    // Build and execute the javac command
    let output = Command::new("javac")
        .arg("-cp")
        .arg(&classpath_str)
        .arg("-d")
        .arg(cls_dir)
        .args(&sources)
        .output()?;

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
    src_dir: &Path,
    class_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let mut to_compile = Vec::new();
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
