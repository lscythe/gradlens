use std::time::Duration;

use crate::model::{ComponentId, ReleaseLink, ReleaseMatch};
use reqwest::Client;
use url::Url;

pub struct ReleaseCandidate {
    pub component: ComponentId,
    pub metadata_urls: Vec<String>,
}

pub struct ReleaseResolver {
    client: Client,
}

impl ReleaseResolver {
    pub fn new() -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: Client::builder()
                .connect_timeout(Duration::from_secs(2))
                .timeout(Duration::from_secs(5))
                .redirect(reqwest::redirect::Policy::limited(5))
                .build()?,
        })
    }

    pub async fn resolve(&self, candidate: &ReleaseCandidate) -> ReleaseLink {
        if let Some(url) = androidx_url(&candidate.component) {
            return ReleaseLink {
                version: candidate.component.version.clone(),
                url: Some(url),
                match_kind: ReleaseMatch::Exact,
                diagnostic: None,
            };
        }
        let repository = candidate
            .metadata_urls
            .iter()
            .find_map(|value| normalize_repository(value));
        let Some(repository) = repository else {
            return none(&candidate.component, None);
        };
        let generic = repository
            .join("releases")
            .ok()
            .or_else(|| Some(repository.clone()));
        let mut diagnostic = None;
        for tag in [
            &candidate.component.version,
            &format!("v{}", candidate.component.version),
        ] {
            for path in [format!("releases/tag/{tag}"), format!("-/releases/{tag}")] {
                let Ok(url) = repository.join(&path) else {
                    continue;
                };
                match self.client.head(url.clone()).send().await {
                    Ok(response) if response.status().is_success() => {
                        return ReleaseLink {
                            version: candidate.component.version.clone(),
                            url: Some(url),
                            match_kind: ReleaseMatch::Exact,
                            diagnostic: None,
                        };
                    }
                    Ok(_) => {}
                    Err(error) => {
                        diagnostic = Some(format!("could not verify exact release: {error}"))
                    }
                }
            }
        }
        ReleaseLink {
            version: candidate.component.version.clone(),
            url: generic,
            match_kind: ReleaseMatch::Generic,
            diagnostic,
        }
    }
}

fn normalize_repository(value: &str) -> Option<Url> {
    let normalized = value
        .strip_prefix("scm:git:")
        .unwrap_or(value)
        .replace("git@github.com:", "https://github.com/")
        .replace("git@gitlab.com:", "https://gitlab.com/");
    let mut url = Url::parse(normalized.trim_end_matches(".git")).ok()?;
    if !matches!(
        url.host_str(),
        Some("github.com" | "gitlab.com" | "127.0.0.1")
    ) {
        return None;
    }
    if !url.path().ends_with('/') {
        url.set_path(&format!("{}/", url.path()));
    }
    Some(url)
}

fn androidx_url(component: &ComponentId) -> Option<Url> {
    let family = component
        .module
        .group
        .strip_prefix("androidx.")?
        .split('.')
        .next()?;
    Url::parse(&format!(
        "https://developer.android.com/jetpack/androidx/releases/{family}#{}",
        component.version
    ))
    .ok()
}

fn none(component: &ComponentId, diagnostic: Option<String>) -> ReleaseLink {
    ReleaseLink {
        version: component.version.clone(),
        url: None,
        match_kind: ReleaseMatch::None,
        diagnostic,
    }
}
