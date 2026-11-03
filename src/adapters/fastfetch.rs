// Fastfetch adapter.
//
// Reads the user's config.jsonc, injects the Murf colors into the
// "logo" and "display" objects, and writes it back.

use super::{home_dir, AppAdapter, ReloadOutcome, WriteOp};
use crate::palette::Palette;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub struct Fastfetch;

impl Fastfetch {
    fn config_path() -> Option<PathBuf> {
        Some(home_dir()?.join(".config").join("fastfetch").join("config.jsonc"))
    }
}

/// Recursively convert jsonc_parser's JsonValue to serde_json::Value.
/// Recursively convert jsonc_parser's JsonValue to serde_json::Value.
fn convert_jsonc(v: jsonc_parser::JsonValue) -> serde_json::Value {
    use jsonc_parser::JsonValue;
    use serde_json::Value;
    match v {
        JsonValue::Null => Value::Null,
        JsonValue::Boolean(b) => Value::Bool(b),
        JsonValue::Number(n) => {
            // In jsonc-parser 0.34, Number is a &str (raw number).
            let parsed: f64 = n.parse().unwrap_or(0.0);
            Value::Number(serde_json::Number::from_f64(parsed).unwrap_or_else(|| 0.into()))
        }
        JsonValue::String(s) => Value::String(s.to_string()),
        JsonValue::Array(a) => Value::Array(a.into_iter().map(convert_jsonc).collect()),
        JsonValue::Object(o) => {
            let mut map = serde_json::Map::new();
            for (k, v) in o {
                map.insert(k.to_string(), convert_jsonc(v));
            }
            Value::Object(map)
        }
    }
}

impl AppAdapter for Fastfetch {
    fn id(&self) -> &'static str { "fastfetch" }
    fn display_name(&self) -> &'static str { "Fastfetch" }

    fn detect(&self, cfg: &crate::config::AppCfg) -> Vec<PathBuf> {
        if let Some(p) = &cfg.config_path {
            let path = PathBuf::from(p);
            if path.exists() { return vec![path]; }
        }
        let dir = match super::home_dir() {
            Some(h) => h.join(".config").join("fastfetch"),
            None => return vec![],
        };
        if !dir.exists() { return vec![]; }
        Self::config_path().into_iter().collect()
    }

    fn plan(&self, palette: &Palette, cfg: &crate::config::AppCfg) -> Result<Vec<WriteOp>> {
        let Some(path) = self.detect(cfg).into_iter().next() else {
            return Ok(vec![]);
        };

        let text = fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string());

        let parsed = jsonc_parser::parse_to_value(
            &text,
            &jsonc_parser::ParseOptions::default(),
        )
        .ok()
        .flatten();

        let mut json = match parsed {
            Some(v) => convert_jsonc(v),
            None => serde_json::json!({}),
        };

        if !json.is_object() {
            json = serde_json::json!({});
        }

        let obj = json.as_object_mut().unwrap();

        let l1 = palette.hex_or("primary",             "#ffffff");
        let l2 = palette.hex_or("secondary",           "#cccccc");
        let l3 = palette.hex_or("tertiary",            "#aaaaaa");
        let l4 = palette.hex_or("primary_container",   "#888888");
        let l5 = palette.hex_or("secondary_container", "#777777");
        let l6 = palette.hex_or("tertiary_container",  "#666666");
        let l7 = palette.hex_or("on_surface",          "#ffffff");
        let l8 = palette.hex_or("outline",             "#888888");
        let l9 = palette.hex_or("error",               "#ff0000");

        let keys      = palette.hex_or("primary",    "#ffffff");
        let title     = palette.hex_or("primary",    "#ffffff");
        let output    = palette.hex_or("on_surface", "#ffffff");
        let separator = palette.hex_or("outline",    "#888888");

        // Inject display.color
        let display = obj.entry("display").or_insert(serde_json::json!({}));
        if let Some(d) = display.as_object_mut() {
            d.insert(
                "color".to_string(),
                serde_json::json!({
                    "keys": keys,
                    "title": title,
                    "output": output,
                    "separator": separator,
                }),
            );
        }

        // Inject logo.color
        let logo = obj.entry("logo").or_insert(serde_json::json!({}));
        if let Some(l) = logo.as_object_mut() {
            l.insert(
                "color".to_string(),
                serde_json::json!({
                    "1": l1, "2": l2, "3": l3, "4": l4, "5": l5,
                    "6": l6, "7": l7, "8": l8, "9": l9,
                }),
            );
        }

        let new_text = serde_json::to_string_pretty(&json)?;

        Ok(vec![WriteOp::File {
            adapter: "fastfetch",
            target: path,
            content: new_text,
            atomic: true,
        }])
    }

    fn reload(&self) -> Result<ReloadOutcome> {
        Ok(ReloadOutcome { deferred: 1, ..Default::default() })
    }
}