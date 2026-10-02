use anyhow::{Result, bail};
use std::path::PathBuf;
use std::process::Command;
use tokio::time::Instant;
use tracing::info;

use crate::{
    cfg,
    io_util::{join_paths, list_files},
    workspace::Workspace,
};

pub async fn run(_ws: &Workspace) -> Result<()> {
    let mut test_deps = list_files(cfg::TEST_DEPENDENCIES_FOLDER).await?;
    test_deps.insert(0, PathBuf::from(cfg::TEST_CLASSES_FOLDER));
    test_deps.insert(0, PathBuf::from(cfg::CLASSES_FOLDER));
    let jar_paths = list_files(cfg::DEPENDENCIES_FOLDER).await?;
    test_deps.extend(jar_paths);

    let classpath_str = join_paths(&test_deps)?;

    let start = Instant::now();

    info!("--------------");

    let mut child = Command::new("java")
        .arg("-cp")
        .arg(&classpath_str)
        .arg("org.junit.platform.console.ConsoleLauncher") //TODO: remove hardcoded class name
        .arg("execute")
        .arg("--scan-class-path")
        .spawn()?; // Use .spawn() to let the program interact with the terminal natively

    let status = child.wait()?;

    info!("--------------");

    if status.success() {
        let elapsed = start.elapsed().as_millis();
        info!("Java application exited successfully, took {}ms", elapsed);
        Ok(())
    } else {
        let status_code = status.code().unwrap_or(-1);
        bail!("Tests failed with exit code: {:?}", status_code);
    }
}
