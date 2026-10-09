// Color math for Murf.
//
// OKLCH conversions use Björn Ottosson's reference formulas.
// Semantic ANSI derivation uses absolute target hues, blended
// toward the seed's hue. Lightness tables differ for dark and
// light palettes, and every derived slot is contrast-checked
// against the palette's background.

pub struct Oklch {
    pub l: f32,
    pub c: f32,
    pub h: f32,
}

pub fn hex_to_oklch(hex: &str) -> Option<Oklch> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 { return None; }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
    Some(srgb_to_oklch(r, g, b))
}

pub fn oklch_to_hex(c: &Oklch) -> String {
    let (r, g, b) = oklch_to_srgb(c);
    let r = (r * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = (g * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = (b * 255.0).round().clamp(0.0, 255.0) as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn srgb_to_oklch(r: f32, g: f32, b: f32) -> Oklch {
    let r = srgb_to_linear(r);
    let g = srgb_to_linear(g);
    let b = srgb_to_linear(b);

    let l_ = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m_ = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s_ = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();

    let l = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
    let a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    let b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;

    let chroma = (a * a + b * b).sqrt();
    let hue = b.atan2(a).to_degrees().rem_euclid(360.0);

    Oklch { l, c: chroma, h: hue }
}

fn oklch_to_srgb(color: &Oklch) -> (f32, f32, f32) {
    let h_rad = color.h.to_radians();
    let a = color.c * h_rad.cos();
    let b = color.c * h_rad.sin();

    let l_ = color.l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = color.l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = color.l - 0.0894841775 * a - 1.2914855480 * b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    let r =  4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

    (linear_to_srgb(r), linear_to_srgb(g), linear_to_srgb(b))
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn linear_to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 { 12.92 * c } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
}

/// WCAG relative luminance of an sRGB color.
pub fn relative_luminance(hex: &str) -> Option<f32> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 { return None; }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
    Some(0.2126 * srgb_to_linear(r)
       + 0.7152 * srgb_to_linear(g)
       + 0.0722 * srgb_to_linear(b))
}

/// WCAG contrast ratio between two colors. Range [1.0, 21.0].
pub fn contrast_ratio(a: &str, b: &str) -> Option<f32> {
    let la = relative_luminance(a)?;
    let lb = relative_luminance(b)?;
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    Some((hi + 0.05) / (lo + 0.05))
}

/// Minimum contrast for a derived ANSI slot against `background`.
pub const MIN_ANSI_CONTRAST: f32 = 3.0;

/// Nudge lightness until the color reaches `min_ratio` against
/// `bg_hex`. Direction depends on `is_dark`: on a dark background,
/// lighten; on a light background, darken. Gives up after 25 steps
/// or when L hits 0.10 or 0.95.
pub fn ensure_readable(hex: &str, bg_hex: &str, is_dark: bool, min_ratio: f32) -> String {
    let Some(mut c) = hex_to_oklch(hex) else { return hex.to_string() };
    let Some(bg_lum) = relative_luminance(bg_hex) else { return hex.to_string() };

    for _ in 0..25 {
        let cur = oklch_to_hex(&c);
        let Some(cur_lum) = relative_luminance(&cur) else { break };
        let (hi, lo) = if cur_lum > bg_lum { (cur_lum, bg_lum) } else { (bg_lum, cur_lum) };
        let ratio = (hi + 0.05) / (lo + 0.05);
        if ratio >= min_ratio { break }
        if is_dark {
            c.l = (c.l + 0.02).min(0.95);
        } else {
            c.l = (c.l - 0.02).max(0.10);
        }
    }
    oklch_to_hex(&c)
}

const BRIGHTEN_CEILING: f32 = 0.92;

pub fn brighten(hex: &str, delta: f32) -> String {
    match hex_to_oklch(hex) {
        Some(mut c) => {
            c.l = (c.l + delta).clamp(0.0, BRIGHTEN_CEILING);
            oklch_to_hex(&c)
        }
        None => hex.to_string(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnsiMode { Semantic, Material }

pub struct AnsiColors {
    pub red: String,
    pub yellow: String,
    pub green: String,
    pub cyan: String,
    pub blue: String,
    pub purple: String,
    pub bright_red: String,
    pub bright_yellow: String,
    pub bright_green: String,
    pub bright_cyan: String,
    pub bright_blue: String,
    pub bright_purple: String,
}

const TARGET_YELLOW: f32 = 95.0;
const TARGET_GREEN:  f32 = 145.0;
const TARGET_CYAN:   f32 = 195.0;
const TARGET_BLUE:   f32 = 250.0;
const TARGET_PURPLE: f32 = 305.0;

const SEED_PULL: f32 = 0.20;

const DARK_TABLE: [(f32, f32); 5] = [
    (0.78, 0.15), // yellow
    (0.62, 0.17), // green
    (0.70, 0.13), // cyan
    (0.55, 0.16), // blue
    (0.55, 0.18), // purple
];

const LIGHT_TABLE: [(f32, f32); 5] = [
    (0.55, 0.16), // yellow
    (0.45, 0.17), // green
    (0.48, 0.14), // cyan
    (0.40, 0.16), // blue
    (0.40, 0.18), // purple
];

fn shortest_hue_delta(from: f32, to: f32) -> f32 {
    let d = (to - from).rem_euclid(360.0);
    if d > 180.0 { d - 360.0 } else { d }
}

fn blend_toward_target(seed_h: f32, target: f32, pull: f32) -> f32 {
    let delta = shortest_hue_delta(seed_h, target);
    (seed_h + delta * (1.0 - pull)).rem_euclid(360.0)
}

pub fn semantic_ansi(
    seed_hex: &str,
    error_hex: &str,
    bg_hex: &str,
    is_dark: bool,
) -> AnsiColors {
    let seed = hex_to_oklch(seed_hex).unwrap_or(Oklch { l: 0.5, c: 0.1, h: 0.0 });
    let h = seed.h;

    let table = if is_dark { &DARK_TABLE } else { &LIGHT_TABLE };

    let make = |target: f32, l: f32, c: f32| -> String {
        let hue = blend_toward_target(h, target, SEED_PULL);
        oklch_to_hex(&Oklch { l, c, h: hue })
    };

    let ensure = |hex: &str| ensure_readable(hex, bg_hex, is_dark, MIN_ANSI_CONTRAST);

    let yellow = ensure(&make(TARGET_YELLOW, table[0].0, table[0].1));
    let green  = ensure(&make(TARGET_GREEN,  table[1].0, table[1].1));
    let cyan   = ensure(&make(TARGET_CYAN,   table[2].0, table[2].1));
    let blue   = ensure(&make(TARGET_BLUE,   table[3].0, table[3].1));
    let purple = ensure(&make(TARGET_PURPLE, table[4].0, table[4].1));
    let red    = ensure(error_hex);

    let bright_yellow = ensure(&brighten(&yellow, 0.10));
    let bright_green  = ensure(&brighten(&green,  0.10));
    let bright_cyan   = ensure(&brighten(&cyan,   0.10));
    let bright_blue   = ensure(&brighten(&blue,   0.10));
    let bright_purple = ensure(&brighten(&purple, 0.10));
    let bright_red    = ensure(&brighten(&red,    0.10));

    AnsiColors {
        red, yellow, green, cyan, blue, purple,
        bright_red, bright_yellow, bright_green,
        bright_cyan, bright_blue, bright_purple,
    }
}

pub fn material_ansi(
    red: &str, green: &str, yellow: &str,
    blue: &str, purple: &str, cyan: &str,
    bright_red: &str, bright_green: &str, bright_yellow: &str,
    bright_blue: &str, bright_purple: &str, bright_cyan: &str,
) -> AnsiColors {
    AnsiColors {
        red: red.to_string(),
        yellow: yellow.to_string(),
        green: green.to_string(),
        cyan: cyan.to_string(),
        blue: blue.to_string(),
        purple: purple.to_string(),
        bright_red: bright_red.to_string(),
        bright_yellow: bright_yellow.to_string(),
        bright_green: bright_green.to_string(),
        bright_cyan: bright_cyan.to_string(),
        bright_blue: bright_blue.to_string(),
        bright_purple: bright_purple.to_string(),
    }
}
