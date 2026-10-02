use anyhow::{Result, bail};
use std::process::Command;
use tokio::time::Instant;
use tracing::info;

use crate::{io_util::build_classpath, workspace::Workspace};

pub fn run(ws: &Workspace) -> Result<()> {
    let classpath_str = build_classpath(
        &[&ws.test_classes_dir(), &ws.classes_dir()],
        &[&ws.test_dependencies_dir(), &ws.dependencies_dir()],
    )?;

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
