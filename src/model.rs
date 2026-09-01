use std::fmt;

pub struct ModuleId {
    pub group: String,
    pub name: String,
}

impl fmt::Display for ModuleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.group, self.name)
    }
}

pub struct ComponentId {
    pub module: ModuleId,
    pub version: String,
}

impl fmt::Display for ComponentId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.module, self.version)
    }
}

pub struct DependencyNode {
    pub component: ComponentId,
    pub children: Vec<DependencyNode>,
    pub cycle: bool,
}

pub enum ReleaseMatch {
    Exact,
    Generic,
    None,
}

pub struct ReleaseLink {
    pub version: String,
    pub url: Option<url::Url>,
    pub match_kind: ReleaseMatch,
    pub diagnostic: Option<String>,
}

pub struct LibraryInspection {
    pub alias: String,
    pub requested: Option<ComponentId>,
    pub selected: ComponentId,
    pub dependencies: Vec<DependencyNode>,
    pub release: ReleaseLink,
}

pub struct Inspection {
    pub configuration: String,
    pub libraries: Vec<LibraryInspection>,
}

#[cfg(test)]
mod tests {
    use super::{ComponentId, ModuleId};

    #[test]
    fn module_id_displays_group_and_name() {
        let module = ModuleId {
            group: "org.example".into(),
            name: "library".into(),
        };

        assert_eq!(module.to_string(), "org.example:library");
    }

    #[test]
    fn component_id_displays_group_name_and_version() {
        let component = ComponentId {
            module: ModuleId {
                group: "org.example".into(),
                name: "library".into(),
            },
            version: "1.2.3".into(),
        };

        assert_eq!(component.to_string(), "org.example:library:1.2.3");
    }
}
