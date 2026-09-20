use anyhow::Result;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use tracing::{error, info};

use crate::deps::Dependency;

pub fn build(dependencies: &[Dependency], base_dir: &str) -> Result<()> {
    // 1. Map your filenames back to their complete local path handles
    /*let jar_paths: Vec<PathBuf> = dependencies
        .iter()
        .map(|filename| PathBuf::from(base_dir).join(filename))
        .collect();
    */

    let mut jar_paths = Vec::new();
    jar_paths.push("fix_me".to_string()); //TODO fix me
    // 2. Safely join paths using the correct OS separator (':' or ';')
    let classpath = env::join_paths(jar_paths)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    let classpath_str = classpath.to_string_lossy();
    info!("Generated classpath: {}", classpath_str);

    // 3. Build and execute the javac system process command
    let output = Command::new("javac")
        .arg("-cp")
        .arg(&*classpath_str) // Passes the formatted classpath string
        .arg("src/main/java/Main.java") // Your target source file(s)
        .arg("-d")
        .arg("target/classes") // Output directory for compiled .class files
        .output()?; // Executes the process and awaits termination

    // 4. Handle stderr/stdout outputs from the compiler
    if output.status.success() {
        info!("Successfully compiled Java source files!");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Java compilation failed:\n{}", stderr);
    }

    Ok(())
}
