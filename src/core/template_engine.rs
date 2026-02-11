use std::collections::HashMap;
use std::path::Path;

use color_eyre::eyre::eyre;
use tera::{Context, Tera};

use crate::config::{Config, GroupConfig};
use crate::result::Result;

const DEFAULT_TEMPLATE: &str = r#"---
id: {{ id }}
title: {{ title }}
---

# {{ title }}

{{ content }}
"#;

/// Resolve which group applies for a given CWD, by checking if CWD starts with
/// any of the group's directories (relative to collection root).
pub fn resolve_group_from_cwd<'a>(
    config: &'a Config,
    collection_root: &Path,
    cwd: &Path,
) -> Option<(&'a str, &'a GroupConfig)> {
    for (name, group_config) in &config.group {
        for dir in &group_config.directories {
            let full_dir = collection_root.join(dir);
            if cwd.starts_with(&full_dir) {
                return Some((name.as_str(), group_config));
            }
        }
    }
    None
}

/// Resolve the template string given resolution priority:
/// 1. Explicit --template flag
/// 2. Group's configured template (from matched group)
/// 3. Hardcoded default template
pub fn resolve_template_string(
    collection_root: &Path,
    template_flag: Option<&str>,
    group_config: Option<&GroupConfig>,
) -> Result<String> {
    // Priority 1: explicit --template flag
    if let Some(tmpl_name) = template_flag {
        return load_template_file(collection_root, tmpl_name);
    }

    // Priority 2: group config template
    if let Some(gc) = group_config {
        if let Some(ref tmpl_name) = gc.template {
            return load_template_file(collection_root, tmpl_name);
        }
    }

    // Priority 3: hardcoded default
    Ok(DEFAULT_TEMPLATE.to_owned())
}

fn load_template_file(collection_root: &Path, name: &str) -> Result<String> {
    let path = if name.contains('.') {
        // treat as a path under .zet/templates/
        collection_root
            .join(format!(".{}", crate::APP_NAME))
            .join("templates")
            .join(name)
    } else {
        // try .zet/templates/<name>.md
        collection_root
            .join(format!(".{}", crate::APP_NAME))
            .join("templates")
            .join(format!("{}.md", name))
    };

    std::fs::read_to_string(&path)
        .map_err(|e| eyre!("could not read template {:?}: {}", path, e))
}

/// Render a template string with the given context variables.
pub fn render_template(
    template_str: &str,
    id: &str,
    title: &str,
    date: &str,
    content: &str,
    extra: &HashMap<String, serde_json::Value>,
) -> Result<String> {
    let mut tera = Tera::default();
    tera.set_escape_fn(|s| s.to_string());
    tera.add_raw_template("note", template_str)
        .map_err(|e| eyre!("failed to parse template: {}", e))?;

    let mut ctx = Context::new();
    ctx.insert("id", id);
    ctx.insert("title", title);
    ctx.insert("date", date);
    ctx.insert("content", content);

    for (key, value) in extra {
        ctx.insert(key.as_str(), value);
    }

    tera.render("note", &ctx)
        .map_err(|e| eyre!("failed to render template: {}", e))
}
