use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{
    catalog,
    changes::{self, ChangeError},
    gradle::{GradleError, GradleInspector},
    graph,
    model::{ChangeKind, Inspection, LibraryInspection},
    releases::{ReleaseCandidate, ReleaseResolver},
};

pub struct Inspector {
    project_root: PathBuf,
    catalog_path: PathBuf,
    releases: ReleaseResolver,
    baseline: Option<String>,
}

#[derive(Debug, Error)]
pub enum InspectError {
    #[error(transparent)]
    Catalog(#[from] catalog::CatalogError),
    #[error(transparent)]
    Gradle(#[from] GradleError),
    #[error("cannot create HTTP client: {0}")]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Change(#[from] ChangeError),
}

impl Inspector {
    pub fn new(
        project_root: impl Into<PathBuf>,
        catalog_path: impl Into<PathBuf>,
        baseline: Option<String>,
    ) -> Result<Self, InspectError> {
        Ok(Self {
            project_root: project_root.into(),
            catalog_path: catalog_path.into(),
            releases: ReleaseResolver::new()?,
            baseline,
        })
    }

    pub fn configurations(&self) -> Result<Vec<String>, InspectError> {
        Ok(GradleInspector::new(&self.project_root).configurations()?)
    }

    pub async fn inspect(&self, selector: &str) -> Result<Inspection, InspectError> {
        let catalog_path = absolute_or_join(&self.project_root, &self.catalog_path);
        let catalog = catalog::parse(&catalog_path)?;
        let changes = self
            .baseline
            .as_deref()
            .map(|baseline| changes::load(&self.project_root, baseline, &catalog_path, &catalog))
            .transpose()?;
        let graph = GradleInspector::new(&self.project_root).resolve(selector)?;
        let resolved = graph::map_used_libraries(&catalog, &graph);
        let mut libraries = Vec::with_capacity(resolved.len());
        for library in resolved {
            let release = self
                .releases
                .resolve(&ReleaseCandidate {
                    component: library.selected.clone(),
                    metadata_urls: library.metadata_urls,
                })
                .await;
            let change = changes
                .as_ref()
                .and_then(|items| items.iter().find(|change| change.alias == library.alias))
                .cloned();
            if changes.is_some() && change.is_none() {
                continue;
            }
            libraries.push(LibraryInspection {
                alias: library.alias,
                requested: library.requested,
                selected: library.selected,
                dependencies: library.dependencies,
                release,
                change,
            });
        }
        let removed = changes
            .unwrap_or_default()
            .into_iter()
            .filter(|change| change.kind == ChangeKind::Removed)
            .collect();
        Ok(Inspection {
            configuration: selector.into(),
            libraries,
            removed,
        })
    }
}

fn absolute_or_join(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.into()
    } else {
        root.join(path)
    }
}
