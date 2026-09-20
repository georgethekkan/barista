use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs};
use tracing::info;

#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub package: PackageConfig,
    // Maps dependency name (e.g., "junit-jupiter-api") to its configuration format
    #[serde(default)]
    pub dependencies: BTreeMap<String, DependencyConfig>,
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)] // Evaluates variants sequentially until one matches the data shape
pub enum DependencyConfig {
    // Variant 1: Matches standard string shortcut -> "org.junit.jupiter:junit-jupiter-api:5.10.0"
    StringNotation(String),

    // Variant 2: Matches the table structure -> { id: "...", exclusions: [...] }
    TableNotation(DependencyDetails),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DependencyDetails {
    // Maps to the "id" key in your inline table
    pub id: String,
    // Fallback safely if a library doesn't list any exclusions
    #[serde(default)]
    pub exclusions: Vec<String>,
}

impl Manifest {
    pub fn dependency_configs(&self) -> Vec<&DependencyConfig> {
        self.dependencies.values().collect::<Vec<_>>()
    }
}

pub fn load() -> Result<Manifest> {
    let cfg_file = "./jb.toml";
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
            java_version = "21"

            [dependencies]
            junit-jupiter-api = "org.junit.jupiter:junit-jupiter-api:5.10.0"
            slf4j-api = { id = "org.slf4j:slf4j-api:2.0.18", exclusions = ["something:something"] }
        "#;

        // Ensure parsing passes without throwing errors
        let manifest: Manifest = toml::from_str(toml_content).unwrap();

        assert_eq!(manifest.package.name, "my-java-app");
        assert_eq!(manifest.package.version, "0.1.0");
        assert_eq!(manifest.package.main_class.as_deref(), Some("Main"));
        assert_eq!(manifest.package.java_version.as_deref(), Some("21"));

        // 1. Verify junit-jupiter-api parsed into the String variant
        match manifest.dependencies.get("junit-jupiter-api") {
            Some(DependencyConfig::StringNotation(coord)) => {
                assert_eq!(coord, "org.junit.jupiter:junit-jupiter-api:5.10.0");
            }
            _ => panic!("Expected junit-jupiter-api to be StringNotation"),
        }

        // 2. Verify slf4j-api parsed into the Table variant with details
        match manifest.dependencies.get("slf4j-api") {
            Some(DependencyConfig::TableNotation(details)) => {
                assert_eq!(details.id, "org.slf4j:slf4j-api:2.0.18");
                assert_eq!(details.exclusions, vec!["something:something".to_string()]);
            }
            _ => panic!("Expected slf4j-api to be TableNotation"),
        }
    }
}
