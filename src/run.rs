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

pub fn run(main_class: &str, app_args: &[String], _ws: &Workspace) -> Result<()> {
    let mut jar_paths = list_files(cfg::DEPENDENCIES_FOLDER)?;
    jar_paths.insert(0, PathBuf::from(cfg::CLASSES_FOLDER));
    let classpath_str = join_paths(&jar_paths)?;

    let start = Instant::now();

    info!("--------------");

    let mut cmd = Command::new("java");
    cmd.arg("-cp")
        .arg(&classpath_str)
        .arg(main_class)
        .args(app_args);

    let mut child = cmd.spawn()?; // Use .spawn() to let the program interact with the terminal natively
    let status = child.wait()?;

    info!("--------------");

    if status.success() {
        let ts = start.elapsed().as_millis();
        info!("Java application exited successfully, took {ts}ms");
        Ok(())
    } else {
        let code = status.code().unwrap_or_default();
        bail!("Java application exited with error code: {code}");
    }
}
