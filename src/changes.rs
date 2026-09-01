use std::{collections::BTreeMap, path::Path, process::Command};

use thiserror::Error;

use crate::{
    catalog::{self, Catalog, CatalogLibrary},
    model::{ChangeKind, ComponentId, LibraryChange},
};

#[derive(Debug, Error)]
pub enum ChangeError {
    #[error("git could not read '{path}' from baseline '{baseline}': {message}")]
    Git {
        baseline: String,
        path: String,
        message: String,
    },
    #[error(transparent)]
    Catalog(#[from] catalog::CatalogError),
}

pub fn load(
    project_root: &Path,
    baseline: &str,
    catalog_path: &Path,
    current: &Catalog,
) -> Result<Vec<LibraryChange>, ChangeError> {
    let relative = catalog_path
        .strip_prefix(project_root)
        .unwrap_or(catalog_path);
    let spec = format!(
        "{baseline}:{}",
        relative.to_string_lossy().replace('\\', "/")
    );
    let output = Command::new("git")
        .current_dir(project_root)
        .args(["show", &spec])
        .output()
        .map_err(|error| ChangeError::Git {
            baseline: baseline.into(),
            path: relative.display().to_string(),
            message: error.to_string(),
        })?;
    if !output.status.success() {
        return Err(ChangeError::Git {
            baseline: baseline.into(),
            path: relative.display().to_string(),
            message: String::from_utf8_lossy(&output.stderr).trim().into(),
        });
    }
    let baseline_catalog = catalog::parse_str(&String::from_utf8_lossy(&output.stdout))?;
    Ok(compare(&baseline_catalog, current))
}

pub fn compare(baseline: &Catalog, current: &Catalog) -> Vec<LibraryChange> {
    let before: BTreeMap<_, _> = baseline
        .libraries
        .iter()
        .map(|library| (&library.alias, library))
        .collect();
    let after: BTreeMap<_, _> = current
        .libraries
        .iter()
        .map(|library| (&library.alias, library))
        .collect();
    before
        .keys()
        .chain(after.keys())
        .fold(BTreeMap::new(), |mut result, alias| {
            let old = before.get(alias).copied();
            let new = after.get(alias).copied();
            let kind = match (old, new) {
                (None, Some(_)) => Some(ChangeKind::Added),
                (Some(_), None) => Some(ChangeKind::Removed),
                (Some(old), Some(new)) if old.module != new.module => {
                    Some(ChangeKind::ModuleChanged)
                }
                (Some(old), Some(new)) if old.requested_version != new.requested_version => {
                    Some(ChangeKind::Updated)
                }
                _ => None,
            };
            if let Some(kind) = kind {
                result.insert(
                    (*alias).clone(),
                    LibraryChange {
                        alias: (*alias).clone(),
                        kind,
                        baseline: old.and_then(component),
                        current: new.and_then(component),
                    },
                );
            }
            result
        })
        .into_values()
        .collect()
}

fn component(library: &CatalogLibrary) -> Option<ComponentId> {
    Some(ComponentId {
        module: library.module.clone(),
        version: library.requested_version.clone()?,
    })
}
