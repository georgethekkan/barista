use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use tracing::{error, info};

use crate::{cfg, deps::Dependency};

pub fn run(dependencies: Vec<Dependency>, main_class: &str) -> std::io::Result<()> {
    // 1. Gather all of your JAR path handles
    /*let mut classpath_paths: Vec<PathBuf> = dependencies
        .iter()
        .map(|filename| PathBuf::from(deps_dir).join(filename))
        .collect();

    // 2. CRITICAL: Append the compiled classes folder path to the classpath array
    classpath_paths.push(PathBuf::from(classes_dir));

    // 3. Safely join everything using the host OS separator (':' on Unix, ';' on Windows)
    let classpath = env::join_paths(classpath_paths)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    let classpath_str = classpath.to_string_lossy();
    info!("Runtime Classpath: {}", classpath_str);
    */

    let classpath_str = cfg::CLASSES_FOLDER;

    // 4. Build and execute the live java process
    let mut child = Command::new("java")
        .arg("-cp")
        .arg(&*classpath_str)
        .arg(main_class) // Example: "com.example.App"
        .spawn()?; // Use .spawn() to let the program interact with the terminal natively

    // 5. Wait for the Java program to finish running
    let status = child.wait()?;

    if status.success() {
        info!("Java application exited successfully.");
    } else {
        error!(
            "Java application exited with an error code: {:?}",
            status.code()
        );
    }

    Ok(())
}
