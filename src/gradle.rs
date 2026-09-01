use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use thiserror::Error;

use crate::model::ComponentId;

const BEGIN: &str = "GRADLE_CHECKER_BEGIN";
const END: &str = "GRADLE_CHECKER_END";
const INIT_SCRIPT: &str = include_str!("gradle/init.gradle.kts");

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResolvedComponent {
    pub id: ComponentId,
    #[serde(default)]
    pub children: Vec<String>,
    #[serde(default)]
    pub metadata_urls: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResolvedGraph {
    pub components: BTreeMap<String, ResolvedComponent>,
    pub roots: Vec<String>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigurationCandidate {
    pub project: String,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum GradleError {
    #[error("Gradle Wrapper not found in {0}")]
    MissingWrapper(String),
    #[error("Gradle failed: {0}")]
    Process(String),
    #[error("inspection output contained no payload")]
    MissingPayload,
    #[error("inspection output contained multiple payloads")]
    DuplicatePayload,
    #[error("invalid inspection payload: {0}")]
    InvalidPayload(#[from] serde_json::Error),
    #[error("inspection payload references unknown child '{0}'")]
    UnknownChild(String),
    #[error("configuration '{0}' was not found")]
    MissingConfiguration(String),
    #[error("configuration '{selector}' is ambiguous: {candidates}")]
    AmbiguousConfiguration {
        selector: String,
        candidates: String,
    },
    #[error("cannot create temporary init script: {0}")]
    Temp(#[from] std::io::Error),
}

pub struct GradleInspector {
    project_root: PathBuf,
}

impl GradleInspector {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
        }
    }

    pub fn configurations(&self) -> Result<Vec<String>, GradleError> {
        let output = self.invoke("list", None)?;
        let payload: ConfigurationPayload = decode_payload(&output)?;
        Ok(payload
            .configurations
            .into_iter()
            .map(|c| format_selector(&c))
            .collect())
    }

    pub fn resolve(&self, selector: &str) -> Result<ResolvedGraph, GradleError> {
        decode_graph(&self.invoke("resolve", Some(selector))?)
    }

    fn invoke(&self, mode: &str, selector: Option<&str>) -> Result<String, GradleError> {
        let wrapper = wrapper_path(&self.project_root)
            .ok_or_else(|| GradleError::MissingWrapper(self.project_root.display().to_string()))?;
        let mut script = NamedTempFile::new()?;
        std::io::Write::write_all(&mut script, INIT_SCRIPT.as_bytes())?;
        let mut command = Command::new(wrapper);
        command
            .current_dir(&self.project_root)
            .arg("--init-script")
            .arg(script.path())
            .arg("--console=plain")
            .arg("gradleCheckerInspect")
            .arg(format!("-PgradleCheckerMode={mode}"));
        if let Some(selector) = selector {
            command.arg(format!("-PgradleCheckerConfiguration={selector}"));
        }
        let output = command.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            return Err(GradleError::Process(if stderr.is_empty() {
                format!("exit status {}", output.status)
            } else {
                stderr
            }));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

#[derive(Deserialize)]
struct ConfigurationPayload {
    configurations: Vec<ConfigurationCandidateWire>,
}
#[derive(Deserialize)]
struct ConfigurationCandidateWire {
    project: String,
    name: String,
}

pub fn decode_graph(output: &str) -> Result<ResolvedGraph, GradleError> {
    let graph: ResolvedGraph = decode_payload(output)?;
    for root in &graph.roots {
        if !graph.components.contains_key(root) {
            return Err(GradleError::UnknownChild(root.clone()));
        }
    }
    for component in graph.components.values() {
        for child in &component.children {
            if !graph.components.contains_key(child) {
                return Err(GradleError::UnknownChild(child.clone()));
            }
        }
    }
    Ok(graph)
}

fn decode_payload<T: for<'de> Deserialize<'de>>(output: &str) -> Result<T, GradleError> {
    let parts: Vec<_> = output.split(BEGIN).skip(1).collect();
    if parts.is_empty() {
        return Err(GradleError::MissingPayload);
    }
    if parts.len() != 1 {
        return Err(GradleError::DuplicatePayload);
    }
    let payload = parts[0]
        .split_once(END)
        .ok_or(GradleError::MissingPayload)?
        .0
        .trim();
    serde_json::from_str(payload).map_err(Into::into)
}

#[allow(dead_code)]
pub fn resolve_selector(
    selector: &str,
    values: &[ConfigurationCandidate],
) -> Result<ConfigurationCandidate, GradleError> {
    let matches: Vec<_> = if selector.starts_with(':') {
        values
            .iter()
            .filter(|value| format_selector(*value) == selector)
            .cloned()
            .collect()
    } else {
        values
            .iter()
            .filter(|value| value.name == selector)
            .cloned()
            .collect()
    };
    match matches.as_slice() {
        [value] => Ok(value.clone()),
        [] => Err(GradleError::MissingConfiguration(selector.into())),
        _ => Err(GradleError::AmbiguousConfiguration {
            selector: selector.into(),
            candidates: matches
                .iter()
                .map(format_selector)
                .collect::<Vec<_>>()
                .join(", "),
        }),
    }
}

fn format_selector(value: &impl Candidate) -> String {
    let project = value.project();
    if project == ":" {
        format!(":{}", value.name())
    } else {
        format!("{project}:{}", value.name())
    }
}
trait Candidate {
    fn project(&self) -> &str;
    fn name(&self) -> &str;
}
impl Candidate for ConfigurationCandidate {
    fn project(&self) -> &str {
        &self.project
    }
    fn name(&self) -> &str {
        &self.name
    }
}
impl Candidate for ConfigurationCandidateWire {
    fn project(&self) -> &str {
        &self.project
    }
    fn name(&self) -> &str {
        &self.name
    }
}

fn wrapper_path(root: &Path) -> Option<PathBuf> {
    let name = if cfg!(windows) {
        "gradlew.bat"
    } else {
        "gradlew"
    };
    let path = root.join(name);
    fs::metadata(&path)
        .ok()
        .filter(|m| m.is_file())
        .map(|_| path)
}
