use anyhow::{Result, bail};
use std::process::Command;
use tokio::time::Instant;
use tracing::{debug, info};

use crate::{io_util::build_classpath, workspace::Workspace};

pub fn run(main_class: &str, app_args: &[String], ws: &Workspace) -> Result<()> {
    let classpath = build_classpath(&[&ws.classes_dir()], &[&ws.dependencies_dir()])?;

    let start = Instant::now();

    info!("--------------");

    let args = app_args.join(" ");
    debug!("Running java -cp {classpath} {main_class} {args}");

    let mut cmd = Command::new("java");
    cmd.arg("-cp").arg(&classpath).arg(main_class).args(app_args);

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
