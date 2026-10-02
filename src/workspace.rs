use crate::manifest::{DependencyConfig, Manifest};
use anyhow::{Context, Result, anyhow, bail};
use ristretto_resolver::ArtifactCoordinate;
use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
};
use tracing::info;

use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exclusion {
    pub group: String,
    pub id: String,
}

impl FromStr for Exclusion {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
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

/// The runtime representation of a dependency. Completely separate from the TOML string configuration.
#[derive(Debug, Clone)]
pub enum ResolvedDependency {
    /// A verified third-party external Maven package
    External {
        name: String,
        coordinate: ArtifactCoordinate,
        exclusions: Vec<Exclusion>,
        local_jar_path: PathBuf,
        file_name: String,
    },
    /// A structured pointer to another internal module inside this workspace
    InternalModule { name: String, absolute_path: PathBuf },
}

#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    pub name: String,
    pub manifest: Manifest,
    pub path: PathBuf,
    pub dependencies: Vec<ResolvedDependency>,
    pub dev_dependencies: Vec<ResolvedDependency>,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root_manifest: Manifest,
    pub root_dir: PathBuf,
    pub dependencies: Vec<ResolvedDependency>,
    pub dev_dependencies: Vec<ResolvedDependency>,
    pub members: BTreeMap<String, WorkspaceMember>,
}

impl Workspace {
    /// Compiles the raw manifests from a top-level root folder into a type-safe graph
    pub fn from_root_dir<P: AsRef<Path>>(root_path: P, config_file: &str) -> Result<Self> {
        let root_dir = root_path.as_ref().to_path_buf();
        let root_toml_path = root_dir.join(config_file);
        info!("reading root manifest from {root_toml_path:?}");
        let root_content = std::fs::read_to_string(&root_toml_path)
            .with_context(|| format!("failed to read root manifest {root_toml_path:?}"))?;
        let root_manifest = crate::manifest::parse(&root_content)?;

        let mut members = BTreeMap::new();
        for member_name in &root_manifest.workspace.members {
            let workspace_member = to_workspace_member(member_name, &root_dir, config_file)?;
            members.insert(member_name.clone(), workspace_member);
        }

        let dependencies = resolve_dependencies(&root_manifest.dependencies, &root_dir)?;
        let dev_dependencies = resolve_dependencies(&root_manifest.dev_dependencies, &root_dir)?;

        Ok(Self {
            root_manifest,
            root_dir,
            dependencies,
            dev_dependencies,
            members,
        })
    }

    /// Returns all unique external runtime dependencies across the root and members
    pub fn all_external_dependencies(&self) -> Vec<ResolvedDependency> {
        extract_unique_external(
            self.dependencies
                .iter()
                .chain(self.members.values().flat_map(|m| &m.dependencies)),
        )
    }

    /// Returns all unique external development dependencies across the root and members
    pub fn all_external_dev_dependencies(&self) -> Vec<ResolvedDependency> {
        extract_unique_external(
            self.dev_dependencies
                .iter()
                .chain(self.members.values().flat_map(|m| &m.dev_dependencies)),
        )
    }

    pub fn resolve_compile_order(&self) -> Vec<&WorkspaceMember> {
        let mut visited = HashSet::new();
        let mut order = Vec::with_capacity(self.members.len());

        fn visit<'a>(
            name: &'a str,
            ws: &'a Workspace,
            visited: &mut HashSet<&'a str>,
            order: &mut Vec<&'a WorkspaceMember>,
        ) {
            if !visited.insert(name) {
                return;
            }
            let Some(member) = ws.members.get(name) else {
                return;
            };

            for dep in &member.dependencies {
                if let ResolvedDependency::InternalModule { name: dep_name, .. } = dep {
                    visit(dep_name, ws, visited, order);
                }
            }
            order.push(member);
        }

        for name in self.members.keys() {
            visit(name, self, &mut visited, &mut order);
        }

        order
    }
}

fn extract_unique_external<'a>(
    deps: impl IntoIterator<Item = &'a ResolvedDependency>,
) -> Vec<ResolvedDependency> {
    let mut seen = HashSet::new();
    deps.into_iter()
        .filter_map(|dep| match dep {
            ResolvedDependency::External { coordinate, .. } => {
                seen.insert(coordinate.to_string()).then(|| dep.clone())
            }
            _ => None,
        })
        .collect()
}

fn to_workspace_member(name: &str, base_dir: &Path, config_file: &str) -> Result<WorkspaceMember> {
    let member_dir = base_dir.join(name);
    let member_toml_path = member_dir.join(config_file);
    let member_content = std::fs::read_to_string(&member_toml_path)?;
    let member_manifest = crate::manifest::parse(&member_content)?;

    let dependencies = resolve_dependencies(&member_manifest.dependencies, &member_dir)?;
    let dev_dependencies = resolve_dependencies(&member_manifest.dev_dependencies, &member_dir)?;

    let workspace_member = WorkspaceMember {
        name: name.to_string(),
        manifest: member_manifest,
        path: member_dir,
        dependencies,
        dev_dependencies,
    };

    Ok(workspace_member)
}

/// Helper block translating text definitions into strict target graph objects
fn resolve_dependencies(
    raw_deps: &BTreeMap<String, DependencyConfig>,
    base_dir: &Path,
) -> Result<Vec<ResolvedDependency>> {
    let mut resolved = Vec::new();

    for (name, config) in raw_deps {
        let resolved_dp = to_resolved_dependency(name, config, base_dir)?;
        resolved.push(resolved_dp);
    }

    Ok(resolved)
}

pub fn to_resolved_dependency(name: &str, config: &DependencyConfig, base_dir: &Path) -> Result<ResolvedDependency> {
    use DependencyConfig::*;
    match config {
        Simple(coord) => resolve_external(name, coord, Vec::new()),

        Detailed {
            id: Some(coord),
            exclusions,
            ..
        } => {
            let parsed_exclusions = exclusions
                .iter()
                .map(|s| s.parse::<Exclusion>())
                .collect::<Result<Vec<_>>>()?;
            resolve_external(name, coord, parsed_exclusions)
        }

        Detailed { path: Some(_), .. } => {
            let absolute_path = config
                .resolve_absolute_path(base_dir)
                .ok_or_else(|| anyhow!("Failed to process path for '{name}'"))?;

            Ok(ResolvedDependency::InternalModule {
                name: name.to_string(),
                absolute_path,
            })
        }

        _ => bail!("Dependency '{name}' must specify either a valid `id` or a local `path`"),
    }
}

fn resolve_external(name: &str, coord_str: &str, exclusions: Vec<Exclusion>) -> Result<ResolvedDependency> {
    let (group, id, version) = to_artifact_coordinate(coord_str)?;

    let coordinate = ArtifactCoordinate::new(group, id, version)?;
    let local_jar_path = Path::new(&coordinate.artifact_path()).to_path_buf();
    let file_name = local_jar_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    Ok(ResolvedDependency::External {
        name: name.into(),
        coordinate,
        exclusions,
        local_jar_path,
        file_name,
    })
}

fn to_artifact_coordinate(coord_str: &str) -> Result<(String, String, String)> {
    let mut split = coord_str.split(':');
    let group = split.next().ok_or_else(|| anyhow!("missing group in `{coord_str}`"))?;
    let id = split.next().ok_or_else(|| anyhow!("missing id in `{coord_str}`"))?;
    let version = split
        .next()
        .ok_or_else(|| anyhow!("missing version in `{coord_str}`"))?;

    if split.next().is_some() {
        bail!("dependency coordinate must contain exactly 3 parts: `{coord_str}`");
    }

    Ok((group.into(), id.into(), version.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_workspace_resolution() {
        let toml_content = r#"
            [package]
            name = "app"

            [dependencies]
            junit-jupiter-api = "org.junit.jupiter:junit-jupiter-api:5.10.0"
            slf4j-api = { id = "org.slf4j:slf4j-api:2.0.18", exclusions = ["something:something"] }
            shared = { path = "../shared" }
        "#;

        let raw_manifest = crate::manifest::parse(toml_content).unwrap();
        let base_dir = Path::new("/workspace/root/app");

        let resolved = resolve_dependencies(&raw_manifest.dependencies, base_dir).unwrap();
        assert_eq!(resolved.len(), 3);

        // Verify the internal module path resolved safely without faking coordinates
        let shared_dep = resolved
            .iter()
            .find(|d| match d {
                ResolvedDependency::InternalModule { name, .. } => name == "shared",
                _ => false,
            })
            .unwrap();

        if let ResolvedDependency::InternalModule { name, absolute_path } = shared_dep {
            assert_eq!(name, "shared");
            // Standardizes path parsing clean out of "../" context loops
            assert_eq!(absolute_path, &PathBuf::from("/workspace/root/shared"));
        } else {
            panic!("Expected an InternalModule variant!");
        }
    }
}
