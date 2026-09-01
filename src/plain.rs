use std::fmt::Write;

use crate::model::{DependencyNode, Inspection, ReleaseMatch};

pub fn render(inspection: &Inspection) -> String {
    let mut output = format!("configuration: {}\n", inspection.configuration);
    for library in &inspection.libraries {
        let _ = writeln!(output, "\n{}", library.alias);
        if let Some(change) = &library.change {
            let _ = writeln!(output, "  change:    {}", change.kind.label());
            let _ = writeln!(
                output,
                "  baseline:  {}",
                change
                    .baseline
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "not present".into())
            );
            let _ = writeln!(
                output,
                "  current:   {}",
                change
                    .current
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "not present".into())
            );
        }
        let requested = library
            .requested
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "managed by Gradle".into());
        let _ = writeln!(output, "  requested: {requested}");
        let _ = writeln!(output, "  selected:  {}", library.selected);
        let _ = writeln!(output, "  release notes:");
        let _ = writeln!(output, "    version: {}", library.release.version);
        let _ = writeln!(
            output,
            "    url: {}",
            library
                .release
                .url
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "not found".into())
        );
        let label = match library.release.match_kind {
            ReleaseMatch::Exact => "exact",
            ReleaseMatch::Generic => "generic",
            ReleaseMatch::None => "none",
        };
        let _ = writeln!(output, "    match: {label}");
        if let Some(diagnostic) = &library.release.diagnostic {
            let _ = writeln!(output, "    note: {diagnostic}");
        }
        let _ = writeln!(output, "  dependencies:");
        for (index, node) in library.dependencies.iter().enumerate() {
            render_node(
                &mut output,
                node,
                "    ",
                index + 1 == library.dependencies.len(),
            );
        }
    }
    if !inspection.removed.is_empty() {
        let _ = writeln!(output, "\nremoved libraries:");
        for change in &inspection.removed {
            let _ = writeln!(
                output,
                "  {}: {}",
                change.alias,
                change
                    .baseline
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "unknown".into())
            );
        }
    }
    output
}

fn render_node(output: &mut String, node: &DependencyNode, prefix: &str, last: bool) {
    let branch = if last { "└──" } else { "├──" };
    let cycle = if node.cycle { " (cycle)" } else { "" };
    let _ = writeln!(output, "{prefix}{branch} {}{cycle}", node.component);
    let child_prefix = format!("{prefix}{}   ", if last { " " } else { "│" });
    for (index, child) in node.children.iter().enumerate() {
        render_node(
            output,
            child,
            &child_prefix,
            index + 1 == node.children.len(),
        );
    }
}
