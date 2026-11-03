// YASB Auto-Theme Parser

use crate::palette::Palette;

pub fn generate_override_block(palette: &Palette) -> String {
    let mut block = String::from("\n/* ===== MURF THEME START ===== */\n");
    
    block.push_str(":root {\n");
    block.push_str(&format!("  --background: {} !important;\n", palette.hex_or("surface", "#000")));
    block.push_str(&format!("  --background2: {} !important;\n", palette.hex_or("surface_container_high", "#000")));
    block.push_str(&format!("  --accent: {} !important;\n", palette.hex_or("primary", "#fff")));
    block.push_str(&format!("  --text: {} !important;\n", palette.hex_or("on_surface", "#fff")));
    block.push_str(&format!("  --accentText: {} !important;\n", palette.hex_or("on_primary", "#000")));
    block.push_str(&format!("  --border: {} !important;\n", palette.hex_or("outline", "#888")));
    block.push_str(&format!("  --hover: {} !important;\n", palette.hex_or("surface_container_highest", "#333")));
    block.push_str(&format!("  --mutedBG: {} !important;\n", palette.hex_or("surface_dim", "#111")));
    block.push_str(&format!("  --redFlash: {} !important;\n", palette.hex_or("error", "#f00")));
    block.push_str(&format!("  --subtext: {} !important;\n", palette.hex_or("secondary", "#ccc")));
    block.push_str("}\n");

    block.push_str("/* ===== MURF THEME END ===== */\n");
    block
}

pub fn strip_block(css: &str) -> String {
    let start_marker = "/* ===== MURF THEME START ===== */";
    let end_marker = "/* ===== MURF THEME END ===== */";
    
    if let Some(start) = css.find(start_marker) {
        if let Some(end) = css.find(end_marker) {
            let end_idx = end + end_marker.len();
            let mut out = css[..start].to_string();
            out.push_str(&css[end_idx..]);
            return out;
        }
    }
    css.to_string()
}