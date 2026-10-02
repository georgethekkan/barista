use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use tracing::info;

use crate::cfg;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Manifest {
    pub package: PackageConfig,

    #[serde(default)]
    pub dependencies: BTreeMap<String, DependencyConfig>,

    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: BTreeMap<String, DependencyConfig>,

    #[serde(default)]
    pub workspace: WorkspaceConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct WorkspaceConfig {
    #[serde(default)]
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageConfig {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub main_class: Option<String>,
    #[serde(default)]
    pub java_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum DependencyConfig {
    // Matches: logback-classic = "ch.qos.logback:logback-classic:1.6.3"
    Simple(String),

    // Matches: slf4j-api = { id = "...", exclusions = [...] }
    // AND:     shared = { path = "../shared" }
    Detailed {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        exclusions: Vec<String>,
        #[serde(default)]
        path: Option<PathBuf>,
    },
}

impl DependencyConfig {
    /// Sanitizes and creates a clean absolute normalized path for local modules
    pub fn resolve_absolute_path(&self, base_dir: &Path) -> Option<PathBuf> {
        if let DependencyConfig::Detailed { path: Some(p), .. } = self {
            let combined = base_dir.join(p);
            Some(normalize_path(&combined))
        } else {
            None
        }
    }
}

pub fn normalize_path(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            c => normalized.push(c.as_os_str()),
        }
    }
    normalized
}

pub fn load() -> Result<Manifest> {
    let cfg_file = cfg::CONFIG_FILE;
    info!("Loading project manifest {}", cfg_file);
    let data = fs::read(cfg_file).context("error loading config")?;
    toml::from_slice(&data).context("error parsing input")
}

pub fn parse(input: &str) -> Result<Manifest> {
    toml::from_str(input).context("error parsing input")
}
