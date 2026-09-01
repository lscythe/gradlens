use std::{collections::BTreeMap, fs, path::Path};

use serde::Deserialize;
use thiserror::Error;

use crate::model::ModuleId;

#[derive(Debug)]
pub struct Catalog {
    pub libraries: Vec<CatalogLibrary>,
}

#[derive(Debug)]
pub struct CatalogLibrary {
    pub alias: String,
    pub module: ModuleId,
    pub requested_version: Option<String>,
}

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("cannot read catalog {path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("invalid catalog TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("library '{alias}': {message}")]
    Library { alias: String, message: String },
}

#[derive(Deserialize)]
struct RawCatalog {
    #[serde(default)]
    versions: BTreeMap<String, VersionValue>,
    #[serde(default)]
    libraries: BTreeMap<String, LibraryValue>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum VersionValue {
    Simple(String),
    Rich(RichVersion),
}

#[derive(Deserialize)]
struct RichVersion {
    strictly: Option<String>,
    require: Option<String>,
    prefer: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum LibraryValue {
    String(String),
    Table(LibraryTable),
}

#[derive(Deserialize)]
struct LibraryTable {
    module: Option<String>,
    group: Option<String>,
    name: Option<String>,
    version: Option<VersionSpec>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum VersionSpec {
    Simple(String),
    Ref {
        #[serde(rename = "ref")]
        reference: String,
    },
    Rich(RichVersion),
}

pub fn parse(path: &Path) -> Result<Catalog, CatalogError> {
    let text = fs::read_to_string(path).map_err(|source| CatalogError::Read {
        path: path.display().to_string(),
        source,
    })?;
    let raw: RawCatalog = toml::from_str(&text)?;
    let mut libraries = Vec::with_capacity(raw.libraries.len());
    for (alias, value) in raw.libraries {
        libraries.push(normalize(alias, value, &raw.versions)?);
    }
    Ok(Catalog { libraries })
}

fn normalize(
    alias: String,
    value: LibraryValue,
    versions: &BTreeMap<String, VersionValue>,
) -> Result<CatalogLibrary, CatalogError> {
    let (module, requested_version) = match value {
        LibraryValue::String(coordinate) => {
            let parts: Vec<_> = coordinate.split(':').collect();
            if parts.len() != 3 || parts.iter().any(|part| part.is_empty()) {
                return library_error(alias, "expected 'group:name:version'");
            }
            (
                ModuleId {
                    group: parts[0].into(),
                    name: parts[1].into(),
                },
                Some(parts[2].into()),
            )
        }
        LibraryValue::Table(table) => {
            let module = match (table.module, table.group, table.name) {
                (Some(module), None, None) => parse_module(&alias, &module)?,
                (None, Some(group), Some(name)) if !group.is_empty() && !name.is_empty() => {
                    ModuleId { group, name }
                }
                _ => return library_error(alias, "expected module or group plus name"),
            };
            let version = match table.version {
                None => None,
                Some(VersionSpec::Simple(value)) => Some(value),
                Some(VersionSpec::Rich(value)) => rich_value(&value),
                Some(VersionSpec::Ref { reference }) => versions
                    .get(&reference)
                    .and_then(version_value)
                    .ok_or_else(|| CatalogError::Library {
                        alias: alias.clone(),
                        message: format!("version reference '{reference}' is missing or empty"),
                    })
                    .map(Some)?,
            };
            (module, version)
        }
    };
    Ok(CatalogLibrary {
        alias,
        module,
        requested_version,
    })
}

fn parse_module(alias: &str, value: &str) -> Result<ModuleId, CatalogError> {
    let parts: Vec<_> = value.split(':').collect();
    if parts.len() != 2 || parts.iter().any(|part| part.is_empty()) {
        return library_error(alias.to_owned(), "expected module 'group:name'");
    }
    Ok(ModuleId {
        group: parts[0].into(),
        name: parts[1].into(),
    })
}

fn version_value(value: &VersionValue) -> Option<String> {
    match value {
        VersionValue::Simple(value) => Some(value.clone()),
        VersionValue::Rich(value) => rich_value(value),
    }
}

fn rich_value(value: &RichVersion) -> Option<String> {
    value
        .strictly
        .clone()
        .or_else(|| value.require.clone())
        .or_else(|| value.prefer.clone())
}

fn library_error<T>(alias: String, message: impl Into<String>) -> Result<T, CatalogError> {
    Err(CatalogError::Library {
        alias,
        message: message.into(),
    })
}
