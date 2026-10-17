// Fastfetch adapter.
//
// Fastfetch only allows one --config. To keep the user's modules and
// ASCII while updating colors, we parse their config.jsonc, inject
// the Murf color objects into logo and display, and write it back.

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

impl AppAdapter for Fastfetch {
    fn id(&self) -> &'static str { "fastfetch" }
    fn display_name(&self) -> &'static str { "Fastfetch" }

    fn detect(&self) -> Vec<PathBuf> {
        match Self::config_path() {
            Some(p) if p.exists() => vec![p],
            _ => vec![],
        }
    }

    fn plan(&self, palette: &Palette) -> Result<Vec<WriteOp>> {
        let Some(path) = Self::config_path() else { return Ok(vec![]); };
        let text = fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string());

        let mut json: serde_json::Value = match jsonc_parser::parse_to_value(&text) {
            Ok(Some(v)) => v,
            _ => serde_json::json!({}),
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
        let separator = palette.hex_or("on_surface", "#ffffff");
        let main      = palette.hex_or("on_surface", "#ffffff");

        let display = obj.entry("display").or_insert(serde_json::json!({}));
        if let Some(d) = display.as_object_mut() {
            d.insert("color".to_string(), serde_json::json!({
                "keys": keys,
                "title": title,
                "output": output,
                "separator": separator,
                "main": main,
            }));
        }

        let logo = obj.entry("logo").or_insert(serde_json::json!({}));
        if let Some(l) = logo.as_object_mut() {
            l.insert("color".to_string(), serde_json::json!({
                "1": l1, "2": l2, "3": l3, "4": l4, "5": l5,
                "6": l6, "7": l7, "8": l8, "9": l9,
            }));
        }

        let new_text = serde_json::to_string_pretty(&json)?;

        Ok(vec![WriteOp::File {
            adapter: "fastfetch",
            target: path,
            content: new_text,
        }])
    }

    fn reload(&self) -> Result<ReloadOutcome> {
        Ok(ReloadOutcome { deferred: 1, ..Default::default() })
    }
}
