use anyhow::{Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use tokio::time::Instant;
use tracing::{info, instrument};

use crate::io_util::{build_classpath, list_files};
use crate::workspace::Workspace;

#[instrument(name = "build_main", skip_all)]
pub fn build(ws: &Workspace) -> Result<()> {
    inner_build(
        ws,
        "src/main/java",
        &ws.classes_dir(),
        &[&ws.dependencies_dir()],
        &[],
    )
}

#[instrument(name = "build_test", skip_all)]
pub fn build_test(ws: &Workspace) -> Result<()> {
    inner_build(
        ws,
        "src/test/java",
        &ws.test_classes_dir(),
        &[&ws.test_dependencies_dir(), &ws.dependencies_dir()],
        &[&ws.classes_dir()],
    )
}

fn inner_build(
    ws: &Workspace,
    rel_src: &str,
    cls_dir: &Path,
    jar_dirs: &[&Path],
    class_dirs: &[&Path],
) -> Result<()> {
    if ws.members.is_empty() {
        let src_dir = ws.root_dir.join(rel_src);
        return compile_target(&src_dir, cls_dir, jar_dirs, class_dirs);
    }

    // Resolve compile order and execute sequentially
    let compile_order = ws.resolve_compile_order();
    for member in compile_order {
        let member_src = member.path.join(rel_src);
        info!(
            "Compiling member: {}, src_folder: {}, target_folder: {}",
            member.name,
            member_src.display(),
            cls_dir.display(),
        );
        compile_target(&member_src, cls_dir, jar_dirs, class_dirs)?;
    }

    Ok(())
}

#[instrument(skip_all)]
fn compile_target(
    src_dir: &Path,
    cls_dir: &Path,
    jar_dirs: &[&Path],
    class_dirs: &[&Path],
) -> Result<()> {
    let start = Instant::now();

    if !src_dir.exists() {
        return Ok(());
    }

    let sources: Vec<PathBuf> = list_files(src_dir)?
        .into_iter()
        .filter(|p| p.extension().is_some_and(|ext| ext == "java"))
        .collect();

    let sources = sources_to_compile(sources, src_dir, cls_dir)?;

    if sources.is_empty() {
        info!("Incremental build: no sources to compile in {}", src_dir.display());
        return Ok(());
    }

    // Ensure output classes directory exists before compiling
    fs::create_dir_all(cls_dir)?;

    let mut all_class_dirs = vec![cls_dir];
    all_class_dirs.extend_from_slice(class_dirs);
    let classpath_str = build_classpath(&all_class_dirs, jar_dirs)?;

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

fn sources_to_compile(
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
