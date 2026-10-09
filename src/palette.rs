// Palette: M3 roles parsed from matugen JSON.

use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Color {
    pub hex: String,
}

#[derive(Debug, Clone)]
pub struct Palette {
    pub roles: BTreeMap<String, Color>,
    /// Whether matugen chose a dark palette (light text on dark bg).
    pub is_dark: bool,
}

impl Palette {
    pub fn hex(&self, role: &str) -> Option<&str> {
        self.roles.get(role).map(|c| c.hex.as_str())
    }

    pub fn hex_or(&self, role: &str, fallback: &str) -> String {
        self.hex(role).unwrap_or(fallback).to_string()
    }
}
