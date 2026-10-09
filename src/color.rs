// Color math for Murf.
//
// OKLCH conversions use Björn Ottosson's reference formulas
// (https://bottosson.github.io/posts/oklab/).
//
// Semantic ANSI derivation: given a seed (primary) hue H, produce
// six hue-rotated ANSI slots that land in recognizable perceptual
// neighborhoods while staying tinted toward the wallpaper.

pub struct Oklch {
    pub l: f32, // 0.0 .. 1.0
    pub c: f32, // 0.0 .. ~0.4
    pub h: f32, // degrees
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
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Increase OKLCH lightness by `delta`, clamp to [0,1].
pub fn brighten(hex: &str, delta: f32) -> String {
    match hex_to_oklch(hex) {
        Some(mut c) => {
            c.l = (c.l + delta).clamp(0.0, 1.0);
            oklch_to_hex(&c)
        }
        None => hex.to_string(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnsiMode {
    Semantic,
    Material,
}

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

/// Derive ANSI slots from the seed's hue.
///
/// Hue offsets are v1 tunable values. See docs/adapter-mappings.md.
/// `red` is not derived; it uses `error`.
pub fn semantic_ansi(seed_hex: &str, error_hex: &str) -> AnsiColors {
    let seed = hex_to_oklch(seed_hex).unwrap_or(Oklch { l: 0.5, c: 0.1, h: 0.0 });
    let h = seed.h;

    let make = |offset: f32, chroma: f32, l: f32| -> String {
        let hue = (h + offset).rem_euclid(360.0);
        oklch_to_hex(&Oklch { l, c: chroma, h: hue })
    };

    let yellow = make(65.0,  0.13, 0.65);
    let green  = make(125.0, 0.14, 0.60);
    let cyan   = make(175.0, 0.10, 0.62);
    let blue   = make(215.0, 0.15, 0.58);
    let purple = make(285.0, 0.14, 0.60);

    AnsiColors {
        red: error_hex.to_string(),
        yellow: yellow.clone(),
        green: green.clone(),
        cyan: cyan.clone(),
        blue: blue.clone(),
        purple: purple.clone(),
        bright_red:    brighten(error_hex, 0.10),
        bright_yellow: brighten(&yellow,   0.10),
        bright_green:  brighten(&green,    0.10),
        bright_cyan:   brighten(&cyan,     0.10),
        bright_blue:   brighten(&blue,     0.10),
        bright_purple: brighten(&purple,   0.10),
    }
}

/// Material mode: direct M3 role assignment. No derivation.
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
