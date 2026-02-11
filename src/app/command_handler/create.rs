use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;

use color_eyre::eyre::eyre;

use zet::core::template_engine::{render_template, resolve_group_from_cwd, resolve_template_string};
use zet::preamble::*;

#[allow(clippy::too_many_arguments)]
pub fn handle_command(
    root: Option<PathBuf>,
    title: String,
    content: Option<String>,
    group: Option<String>,
    template: Option<String>,
    stdin: bool,
    data_json: Option<String>,
    data_toml: Option<String>,
    data_json_path: Option<PathBuf>,
    data_toml_path: Option<PathBuf>,
) -> Result<()> {
    // Validate stdin and content are mutually exclusive
    if stdin && content.is_some() {
        return Err(eyre!(
            "--stdin and a positional content argument are mutually exclusive"
        ));
    }

    // Resolve collection root
    let collection_root = zet::core::resolve_root(root)?;

    // Load config
    let config = zet::config::Config::resolve(&collection_root)?;

    // Read content from stdin or positional arg
    let body = if stdin {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        content.unwrap_or_default()
    };

    // Resolve group: explicit --group lookup, or CWD-based matching
    let cwd = std::path::absolute(std::env::current_dir()?)?;

    let resolved_group = if let Some(ref group_name) = group {
        let gc = config
            .group
            .get(group_name)
            .ok_or_else(|| eyre!("group '{}' not found in config", group_name))?;
        Some((group_name.as_str(), gc))
    } else {
        resolve_group_from_cwd(&config, &collection_root, &cwd)
    };

    // Resolve template string
    let template_str = resolve_template_string(
        &collection_root,
        template.as_deref(),
        resolved_group.map(|(_, gc)| gc),
    )?;

    // Compute slug, filename, and id
    let slug = zet::core::slug::slugify(&title);
    let filename = format!("{}.md", slug);
    let id = slug.clone();

    // Determine output directory
    let output_dir = if let Some((_, gc)) = resolved_group {
        // Use explicit --group: use group's first directory relative to collection root
        if let Some(dir) = gc.directories.first() {
            let dir_path = collection_root.join(dir);
            std::fs::create_dir_all(&dir_path)?;
            dir_path
        } else {
            cwd.clone()
        }
    } else {
        cwd.clone()
    };

    // But if --group was explicitly provided, use that group's dir; otherwise use CWD
    // (already handled above)

    let output_path = output_dir.join(&filename);

    // Error if file already exists
    if output_path.exists() {
        return Err(eyre!("file already exists: {:?}", output_path));
    }

    // Merge extra data from --data-* flags
    let mut extra: HashMap<String, serde_json::Value> = HashMap::new();

    if let Some(json_str) = data_json {
        let val: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| eyre!("failed to parse --data-json: {}", e))?;
        merge_json_object(&mut extra, val)?;
    }

    if let Some(toml_str) = data_toml {
        let val: toml::Value = toml::from_str(&toml_str)
            .map_err(|e| eyre!("failed to parse --data-toml: {}", e))?;
        let json_val = toml_to_json(val);
        merge_json_object(&mut extra, json_val)?;
    }

    if let Some(path) = data_json_path {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| eyre!("failed to read --data-json-path {:?}: {}", path, e))?;
        let val: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| eyre!("failed to parse JSON from {:?}: {}", path, e))?;
        merge_json_object(&mut extra, val)?;
    }

    if let Some(path) = data_toml_path {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| eyre!("failed to read --data-toml-path {:?}: {}", path, e))?;
        let val: toml::Value = toml::from_str(&content)
            .map_err(|e| eyre!("failed to parse TOML from {:?}: {}", path, e))?;
        let json_val = toml_to_json(val);
        merge_json_object(&mut extra, json_val)?;
    }

    // Build date string (today as %Y-%m-%d)
    let date = jiff::Zoned::now().strftime("%Y-%m-%d").to_string();

    // Render template
    let rendered = render_template(&template_str, &id, &title, &date, &body, &extra)?;

    // Write to file
    std::fs::write(&output_path, rendered)?;

    // Print absolute file path to stdout
    let abs_path = std::path::absolute(&output_path)?;
    println!("{}", abs_path.display());

    Ok(())
}

fn merge_json_object(
    target: &mut HashMap<String, serde_json::Value>,
    val: serde_json::Value,
) -> Result<()> {
    match val {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                target.insert(k, v);
            }
            Ok(())
        }
        _ => Err(eyre!("data must be a JSON/TOML object (key-value map)")),
    }
}

fn toml_to_json(val: toml::Value) -> serde_json::Value {
    match val {
        toml::Value::String(s) => serde_json::Value::String(s),
        toml::Value::Integer(i) => serde_json::Value::Number(i.into()),
        toml::Value::Float(f) => serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        toml::Value::Boolean(b) => serde_json::Value::Bool(b),
        toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(toml_to_json).collect())
        }
        toml::Value::Table(table) => {
            let map = table
                .into_iter()
                .map(|(k, v)| (k, toml_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
    }
}
