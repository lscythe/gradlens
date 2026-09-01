#[path = "../src/model.rs"]
mod model;
#[path = "../src/releases.rs"]
mod releases;

use model::{ComponentId, ModuleId, ReleaseMatch};
use releases::{ReleaseCandidate, ReleaseResolver};

fn component(group: &str, name: &str, version: &str) -> ComponentId {
    ComponentId {
        module: ModuleId {
            group: group.into(),
            name: name.into(),
        },
        version: version.into(),
    }
}

#[tokio::test]
async fn missing_metadata_has_no_release_link() {
    let resolver = ReleaseResolver::new().unwrap();
    let result = resolver
        .resolve(&ReleaseCandidate {
            component: component("x", "y", "1.0"),
            metadata_urls: vec![],
        })
        .await;
    assert_eq!(result.match_kind, ReleaseMatch::None);
    assert!(result.url.is_none());
}

#[tokio::test]
async fn androidx_uses_selected_version_anchor() {
    let resolver = ReleaseResolver::new().unwrap();
    let result = resolver
        .resolve(&ReleaseCandidate {
            component: component("androidx.room", "room-runtime", "2.8.0"),
            metadata_urls: vec![],
        })
        .await;
    assert_eq!(result.match_kind, ReleaseMatch::Exact);
    assert_eq!(
        result.url.unwrap().as_str(),
        "https://developer.android.com/jetpack/androidx/releases/room#2.8.0"
    );
}

#[tokio::test]
async fn unreachable_repository_falls_back_to_generic() {
    let resolver = ReleaseResolver::new().unwrap();
    let result = resolver
        .resolve(&ReleaseCandidate {
            component: component("x", "y", "1.0"),
            metadata_urls: vec!["http://127.0.0.1:9/org/repo".into()],
        })
        .await;
    assert_eq!(result.match_kind, ReleaseMatch::Generic);
    assert!(result.diagnostic.is_some());
}
