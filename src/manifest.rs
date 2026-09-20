use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use ristretto_resolver::ArtifactCoordinate;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::cfg;

#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub package: PackageConfig,
    // Maps dependency name (e.g., "junit-jupiter-api") to its resolved Dependency
    #[serde(default)]
    pub dependencies: BTreeMap<String, Dependency>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackageConfig {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub main_class: Option<String>,
    #[serde(default)]
    pub java_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct Exclusion {
    pub group: String,
    pub id: String,
}

impl TryFrom<String> for Exclusion {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut parts = value.split(':');
        let (Some(group), Some(id)) = (parts.next(), parts.next()) else {
            bail!("expected exclusion in the form `group:id`, got `{value}`");
        };

        if parts.next().is_some() {
            bail!("expected exclusion in the form `group:id`, got `{value}`");
        }

        Ok(Self {
            group: group.to_owned(),
            id: id.to_owned(),
        })
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawDependency {
    String(String),
    Table {
        id: String,
        #[serde(default)]
        exclusions: Vec<Exclusion>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "RawDependency")]
pub struct Dependency {
    pub group: String,
    pub id: String,
    pub version: String,
    pub kind: String,
    pub exclusions: Vec<Exclusion>,
    pub coordinate: ArtifactCoordinate,
    pub path: PathBuf,
    pub file: String,
}

impl Dependency {
    pub fn new(
        group: &str,
        id: &str,
        version: &str,
        kind: &str,
        exclusions: Vec<Exclusion>,
    ) -> Result<Self> {
        let coordinate = ArtifactCoordinate::new(group, id, version)?;
        let path = Path::new(&coordinate.artifact_path()).to_path_buf();

        let file = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let dep = Self {
            group: group.into(),
            id: id.into(),
            version: version.into(),
            kind: kind.into(),
            exclusions,
            coordinate,
            path,
            file,
        };

        Ok(dep)
    }
}

impl TryFrom<RawDependency> for Dependency {
    type Error = anyhow::Error;

    fn try_from(raw: RawDependency) -> Result<Self, Self::Error> {
        let (id_str, exclusions) = match raw {
            RawDependency::String(s) => (s, Vec::new()),
            RawDependency::Table { id, exclusions } => (id, exclusions),
        };

        let mut split = id_str.split(':');
        let group = split
            .next()
            .ok_or_else(|| anyhow!("missing group in `{id_str}`"))?;
        let id = split
            .next()
            .ok_or_else(|| anyhow!("missing id in `{id_str}`"))?;
        let version = split
            .next()
            .ok_or_else(|| anyhow!("missing version in `{id_str}`"))?;

        if split.next().is_some() {
            bail!("dependency must contain exactly 3 parts: `{id_str}`");
        }

        Dependency::new(group, id, version, "none", exclusions)
    }
}

impl Manifest {
    pub fn dependencies(&self) -> Vec<Dependency> {
        self.dependencies.values().cloned().collect()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_java_dependencies() {
        let toml_content = r#"
            [package]
            name = "my-java-app"
            version = "0.1.0"
            main_class = "Main"
            java_version = "25"

            [dependencies]
            junit-jupiter-api = "org.junit.jupiter:junit-jupiter-api:5.10.0"
            slf4j-api = { id = "org.slf4j:slf4j-api:2.0.18", exclusions = ["something:something"] }
        "#;

        let manifest: Manifest = toml::from_str(toml_content).unwrap();

        assert_eq!(manifest.package.name, "my-java-app");
        assert_eq!(manifest.package.version, "0.1.0");
        assert_eq!(manifest.package.main_class.as_deref(), Some("Main"));
        assert_eq!(manifest.package.java_version.as_deref(), Some("21"));

        // 1. Verify junit-jupiter-api parsed directly into Dependency
        let junit = manifest.dependencies.get("junit-jupiter-api").unwrap();
        assert_eq!(junit.group, "org.junit.jupiter");
        assert_eq!(junit.id, "junit-jupiter-api");
        assert_eq!(junit.version, "5.10.0");
        assert!(junit.exclusions.is_empty());

        // 2. Verify slf4j-api parsed with details and exclusions
        let slf4j = manifest.dependencies.get("slf4j-api").unwrap();
        assert_eq!(slf4j.group, "org.slf4j");
        assert_eq!(slf4j.id, "slf4j-api");
        assert_eq!(slf4j.version, "2.0.18");
        assert_eq!(
            slf4j.exclusions,
            vec![Exclusion {
                group: "something".to_string(),
                id: "something".to_string(),
            }]
        );
    }
}
