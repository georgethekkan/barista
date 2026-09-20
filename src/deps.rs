use std::path::{Path, PathBuf};

use anyhow::{Ok, Result, anyhow, bail};
use ristretto_resolver::ArtifactCoordinate;
use tracing::info;

use crate::manifest::DependencyConfig;

pub fn to_dependencies(dependencies: Vec<&DependencyConfig>) -> Result<Vec<Dependency>> {
    let deps = dependencies
        .iter()
        .map(|it| to_dependency(it))
        .collect::<Result<Vec<_>>>()?;

    info!("Found {} dependencies", deps.len());

    Ok(deps)
}

fn to_dependency(dp_cfg: &DependencyConfig) -> Result<Dependency> {
    use DependencyConfig::*;
    let (txt, kind, ex) = match dp_cfg {
        StringNotation(txt) => (txt.to_owned(), "none", vec![]),
        TableNotation(dp) => (dp.id.to_owned(), "none", to_exclusions(&dp.exclusions)?),
    };

    let mut split = txt.split(':');

    let group = split.next().ok_or_else(|| anyhow!("missing group"))?;
    let id = split.next().ok_or_else(|| anyhow!("missing id"))?;
    let version = split.next().ok_or_else(|| anyhow!("missing version"))?;

    if split.next().is_some() {
        bail!("dependency must contain exactly 3 parts: {txt}");
    }

    Dependency::new(group, id, version, kind, ex)
}

fn to_exclusions(exclusions: &[String]) -> Result<Vec<Exclusion>> {
    let res = exclusions
        .iter()
        .map(|it| it.as_str().try_into())
        .collect::<Result<Vec<_>>>()?;
    Ok(res)
}

impl TryFrom<&str> for Exclusion {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut parts = value.split(":");
        let (Some(group), Some(id)) = (parts.next(), parts.next()) else {
            bail!("expected exclusion in the form `group:id`");
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

#[derive(Debug)]
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
        let path = coordinate.artifact_path();
        let path = Path::new(&path).to_path_buf();

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

#[derive(Debug)]
pub struct Exclusion {
    pub group: String,
    pub id: String,
}
