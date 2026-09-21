use anyhow::{Result, bail};
use std::path::PathBuf;
use std::process::Command;
use tokio::time::Instant;
use tracing::info;

use crate::{
    cfg,
    io_util::{join_paths, list_files},
    manifest::Dependency,
};

pub async fn run(_dependencies: Vec<Dependency>, main_class: &str) -> Result<()> {
    let mut jar_paths = list_files(cfg::DEPENDENCIES_FOLDER).await?;
    jar_paths.insert(0, PathBuf::from(cfg::CLASSES_FOLDER));
    let classpath_str = join_paths(&jar_paths)?;

    let start = Instant::now();

    info!("--------------");

    let mut child = Command::new("java")
        .arg("-cp")
        .arg(&classpath_str)
        .arg(main_class) // Example: "com.example.App"
        .spawn()?; // Use .spawn() to let the program interact with the terminal natively

    let status = child.wait()?;

    info!("--------------");

    if status.success() {
        info!(
            "Java application exited successfully, took {}ms",
            start.elapsed().as_millis()
        );
        Ok(())
    } else {
        bail!("Java application exited with error code: {:?}", status.code());
    }
}
