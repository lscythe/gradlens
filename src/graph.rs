use std::collections::{BTreeSet, HashMap, HashSet};

use crate::{
    catalog::Catalog,
    gradle::{ResolvedComponent, ResolvedGraph},
    model::{ComponentId, DependencyNode},
};

#[derive(Clone, Debug)]
pub struct ResolvedLibrary {
    pub alias: String,
    pub requested: Option<ComponentId>,
    pub selected: ComponentId,
    pub dependencies: Vec<DependencyNode>,
    pub metadata_urls: Vec<String>,
}

pub fn map_used_libraries(catalog: &Catalog, graph: &ResolvedGraph) -> Vec<ResolvedLibrary> {
    let by_module: HashMap<_, _> = graph
        .components
        .iter()
        .map(|(key, value)| (value.id.module.clone(), (key, value)))
        .collect();
    let mut libraries: Vec<_> = catalog
        .libraries
        .iter()
        .filter_map(|library| {
            let (key, selected) = by_module.get(&library.module)?;
            let mut path = HashSet::from([(*key).clone()]);
            let dependencies = children(selected, graph, &mut path);
            Some(ResolvedLibrary {
                alias: library.alias.clone(),
                requested: library
                    .requested_version
                    .as_ref()
                    .map(|version| ComponentId {
                        module: library.module.clone(),
                        version: version.clone(),
                    }),
                selected: selected.id.clone(),
                dependencies,
                metadata_urls: selected.metadata_urls.clone(),
            })
        })
        .collect();
    libraries.sort_by(|a, b| a.alias.cmp(&b.alias));
    libraries
}

fn children(
    component: &ResolvedComponent,
    graph: &ResolvedGraph,
    path: &mut HashSet<String>,
) -> Vec<DependencyNode> {
    let keys: BTreeSet<_> = component.children.iter().collect();
    let mut nodes: Vec<_> = keys
        .into_iter()
        .filter_map(|key| {
            let child = graph.components.get(key)?;
            let cycle = !path.insert(key.clone());
            let descendants = if cycle {
                Vec::new()
            } else {
                let result = children(child, graph, path);
                path.remove(key);
                result
            };
            Some(DependencyNode {
                component: child.id.clone(),
                children: descendants,
                cycle,
            })
        })
        .collect();
    nodes.sort_by(|a, b| a.component.cmp(&b.component));
    nodes
}
