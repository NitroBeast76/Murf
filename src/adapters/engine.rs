// The Core Adapter Engine (TOML + Tera)

#![allow(dead_code)] // TODO: remove once the apply pipeline uses every field and method

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use tera::{Kwargs, State, Tera, Value};

#[derive(Debug, Clone, Deserialize)]
pub struct AdapterConfig {
    pub adapter: AdapterMeta,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdapterMeta {
    pub name: String,
    pub target_file: Vec<String>,
    pub apply_strategy: String,
    pub comment_style: String, // slash, hash, dash, html, css
    pub reload: Option<Vec<String>>,
    #[serde(default)]
    pub create_if_missing: bool,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub template_file: Option<String>,
}

pub struct EngineAdapter {
    pub config: AdapterConfig,
    pub tera: Tera,
}

/// `{{ color | rgba(alpha=0.5) }}` -> `rgba(r, g, b, 0.5)`
fn rgba_filter(value: &Value, kwargs: Kwargs, _state: &State) -> std::result::Result<Value, tera::Error> {
    let hex = value.as_str().unwrap_or("#000000").trim_start_matches('#');

    let alpha = match kwargs.get::<Value>("alpha") {
        Ok(Some(v)) => v.as_f64().unwrap_or(1.0),
        _ => 1.0,
    };

    if hex.len() >= 6 && hex.is_ascii() {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        Ok(Value::from(format!("rgba({r}, {g}, {b}, {alpha})")))
    } else {
        Ok(value.clone())
    }
}

/// Strip the common leading whitespace from every non-blank line.
fn dedent(s: &str) -> String {
    let indent = s
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    s.lines()
        .map(|l| l.get(indent..).unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

impl EngineAdapter {
    /// Load an adapter from a .toml string and its template.
    pub fn new(toml_content: &str, template_content: Option<&str>) -> Result<Self> {
        let config: AdapterConfig =
            toml::from_str(toml_content).context("Failed to parse adapter TOML")?;

        // Tera sandboxing: never use Tera::new(), which loads templates from disk.
        let mut tera = Tera::default();

        // Register filters FIRST: Tera 2 validates filter names when a template is compiled.
        tera.register_filter("rgba", rgba_filter);

        let template_str = if let Some(tmpl) = &config.adapter.template {
            tmpl.as_str()
        } else if let Some(tmpl) = template_content {
            tmpl
        } else {
            anyhow::bail!("Adapter {} has no template", config.adapter.name);
        };

        tera.add_raw_template(&config.adapter.name, template_str)?;

        Ok(Self { config, tera })
    }

    /// Render the template using a Palette context.
    pub fn render(&self, palette: &crate::palette::Palette) -> Result<String> {
        let mut context = tera::Context::new();

        for (role, color) in &palette.roles {
            // Owned key: Context::insert needs 'static, not a borrow of `palette`.
            context.insert(role.clone(), &color.hex);
        }
        context.insert("is_dark", &palette.is_dark);

        let rendered = self
            .tera
            .render(&self.config.adapter.name, &context)
            .context("Failed to render Tera template")?;

        Ok(rendered)
    }

    /// Apply the inject strategy: replace text between markers, or append a new block.
    pub fn apply(&self, original: &str, rendered: &str) -> Result<String> {
        let (begin_marker, end_marker) = match self.config.adapter.comment_style.as_str() {
            "slash" => ("// MURF:BEGIN v1", "// MURF:END v1"),
            "hash" => ("# MURF:BEGIN v1", "# MURF:END v1"),
            "dash" => ("-- MURF:BEGIN v1", "-- MURF:END v1"),
            "html" => ("<!-- MURF:BEGIN v1 -->", "<!-- MURF:END v1 -->"),
            "css" => ("/* MURF:BEGIN v1 */", "/* MURF:END v1 */"),
            other => anyhow::bail!("Unsupported comment_style: {other}"),
        };

        // Strip markers in case the template included them (prevents double-marker
        // corruption), then dedent BEFORE trimming so every line loses the same indent.
        let clean_rendered = dedent(&rendered.replace(begin_marker, "").replace(end_marker, ""))
            .trim()
            .to_string();

        let begin_idx = original.find(begin_marker);
        let end_idx = original.find(end_marker);

        match (begin_idx, end_idx) {
            (Some(b), Some(e)) => {
                if b >= e {
                    anyhow::bail!("MURF:END found before MURF:BEGIN");
                }
                if original[b + begin_marker.len()..].contains(begin_marker) {
                    anyhow::bail!("Multiple MURF:BEGIN markers found. Please clean the file manually.");
                }

                let mut new_content = String::new();
                new_content.push_str(&original[..b]);
                new_content.push_str(begin_marker);
                new_content.push('\n');
                new_content.push_str(&clean_rendered);
                new_content.push('\n');
                new_content.push_str(end_marker);
                new_content.push_str(&original[e + end_marker.len()..]);

                Ok(new_content)
            }
            (None, None) => {
                let mut new_content = original.to_string();
                if !new_content.is_empty() && !new_content.ends_with('\n') {
                    new_content.push('\n');
                }
                if !new_content.is_empty() {
                    new_content.push('\n');
                }
                new_content.push_str(begin_marker);
                new_content.push('\n');
                new_content.push_str(&clean_rendered);
                new_content.push('\n');
                new_content.push_str(end_marker);
                new_content.push('\n');

                Ok(new_content)
            }
            _ => anyhow::bail!("Found only one of the MURF markers (either BEGIN or END is missing)."),
        }
    }

    /// Resolve the target path by expanding ${VAR} and finding the first existing file.
    pub fn resolve_target(&self) -> Option<PathBuf> {
        for target in &self.config.adapter.target_file {
            let mut expanded = target.clone();
            for (key, val) in std::env::vars() {
                expanded = expanded.replace(&format!("${{{}}}", key), &val);
            }

            let path = PathBuf::from(&expanded);
            if path.exists() {
                return Some(path);
            }
        }
        None
    }
}

/// Load all embedded TOML adapters.
pub fn load_builtin_adapters() -> Vec<EngineAdapter> {
    let mut adapters = Vec::new();

    // We use include_str! so the adapter is baked into the .exe
    let glazewm_toml = include_str!("definitions/glazewm.toml");
    
    if let Ok(adapter) = EngineAdapter::new(glazewm_toml, None) {
        adapters.push(adapter);
    } else {
        tracing::error!("Failed to load GlazeWM adapter");
    }

    // As you add more .toml files, just add them here:
    // let starship_toml = include_str!("definitions/starship.toml");
    // if let Ok(adapter) = EngineAdapter::new(starship_toml, None) { adapters.push(adapter); }

    adapters
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{Color, Palette};
    use std::collections::BTreeMap;

    fn test_engine() -> EngineAdapter {
        let toml = r#"
            [adapter]
            name = "test_app"
            target_file = ["dummy.txt"]
            apply_strategy = "inject"
            comment_style = "slash"
            template = """
            background: {{ background }};
            primary: {{ primary | rgba(alpha=0.5) }};
            """
        "#;
        EngineAdapter::new(toml, None).unwrap()
    }

    fn test_palette() -> Palette {
        let mut roles = BTreeMap::new();
        roles.insert("background".to_string(), Color { hex: "#121413".into() });
        roles.insert("primary".to_string(), Color { hex: "#853d11".into() });
        Palette { roles, is_dark: true }
    }

    #[test]
    fn test_engine_inject() {
        let engine = test_engine();
        let rendered = engine.render(&test_palette()).unwrap();

        assert!(rendered.contains("background: #121413;"));
        assert!(rendered.contains("primary: rgba(133, 61, 17, 0.5);"));

        // No existing markers: block is appended, user content untouched.
        let original = "user_config: true\n";
        let applied = engine.apply(original, &rendered).unwrap();
        assert!(applied.starts_with("user_config: true\n"));
        assert!(applied.contains("// MURF:BEGIN v1"));
        assert!(applied.contains("// MURF:END v1"));
        // Indentation is normalized across all lines.
        assert!(applied.contains("background: #121413;\nprimary: rgba(133, 61, 17, 0.5);"));

        // Existing markers: old block replaced, surrounding content preserved.
        let old_file = "user_config: true\n// MURF:BEGIN v1\nbackground: #000;\n// MURF:END v1\nother: true\n";
        let updated = engine.apply(old_file, &rendered).unwrap();
        assert!(!updated.contains("background: #000;"));
        assert!(updated.contains("user_config: true"));
        assert!(updated.contains("other: true"));

        // Idempotent.
        assert_eq!(updated, engine.apply(&updated, &rendered).unwrap());
    }

    #[test]
    fn test_apply_rejects_broken_markers() {
        let engine = test_engine();
        let rendered = engine.render(&test_palette()).unwrap();

        // Only BEGIN.
        assert!(engine.apply("// MURF:BEGIN v1\nfoo\n", &rendered).is_err());
        // Only END.
        assert!(engine.apply("foo\n// MURF:END v1\n", &rendered).is_err());
        // END before BEGIN.
        assert!(engine.apply("// MURF:END v1\n// MURF:BEGIN v1\n", &rendered).is_err());
    }
}