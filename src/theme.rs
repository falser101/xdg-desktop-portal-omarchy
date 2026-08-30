use crate::paths::current_theme_dir;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub struct OmarchyTheme {
    pub mode: ColorScheme,
    pub accent: [f64; 3],
    pub background: [u8; 3],
    pub foreground: [u8; 3],
    pub panel: [u8; 3],
    pub muted: [u8; 3],
    pub red: [u8; 3],
    pub accent_rgb: [u8; 3],
    pub source: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ColorScheme {
    NoPreference = 0,
    PreferDark = 1,
    PreferLight = 2,
}

impl Default for OmarchyTheme {
    fn default() -> Self {
        Self {
            mode: ColorScheme::PreferDark,
            accent: [0.537, 0.706, 0.980],
            background: [0x1e, 0x1e, 0x2e],
            foreground: [0xcd, 0xd6, 0xf4],
            panel: [0x31, 0x32, 0x44],
            muted: [0x58, 0x5b, 0x70],
            red: [0xf3, 0x8b, 0xa8],
            accent_rgb: [0x89, 0xb4, 0xfa],
            source: PathBuf::new(),
        }
    }
}

impl OmarchyTheme {
    pub fn load() -> Self {
        let dir = current_theme_dir();
        Self::load_from(&dir.join("colors.toml")).unwrap_or_else(|_| Self {
            source: dir.join("colors.toml"),
            ..Self::default()
        })
    }

    pub fn load_from(path: &Path) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let value: toml::Value = text.parse()?;
        Ok(Self::from_toml(&value, path.to_path_buf()))
    }

    fn from_toml(value: &toml::Value, source: PathBuf) -> Self {
        let get = |key: &str| {
            value
                .get(key)
                .and_then(|v| v.as_str())
                .map(str::to_string)
        };

        let mode = match get("mode")
            .or_else(|| get("color-scheme"))
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "light" => ColorScheme::PreferLight,
            "dark" => ColorScheme::PreferDark,
            _ => ColorScheme::PreferDark,
        };

        let accent_rgb = parse_hex(&get("accent").unwrap_or_else(|| "#89b4fa".into()))
            .unwrap_or([0x89, 0xb4, 0xfa]);
        let background =
            parse_hex(&get("background").unwrap_or_else(|| "#1e1e2e".into())).unwrap_or([0x1e, 0x1e, 0x2e]);
        let foreground =
            parse_hex(&get("foreground").unwrap_or_else(|| "#cdd6f4".into())).unwrap_or([0xcd, 0xd6, 0xf4]);
        let panel = parse_hex(
            &get("lighter_background")
                .or_else(|| get("panel"))
                .unwrap_or_else(|| "#313244".into()),
        )
        .unwrap_or([0x31, 0x32, 0x44]);
        let muted = parse_hex(
            &get("muted")
                .or_else(|| get("dark_foreground"))
                .unwrap_or_else(|| "#585b70".into()),
        )
        .unwrap_or([0x58, 0x5b, 0x70]);
        let red = parse_hex(&get("red").unwrap_or_else(|| "#f38ba8".into())).unwrap_or([0xf3, 0x8b, 0xa8]);

        Self {
            mode,
            accent: [
                accent_rgb[0] as f64 / 255.0,
                accent_rgb[1] as f64 / 255.0,
                accent_rgb[2] as f64 / 255.0,
            ],
            background,
            foreground,
            panel,
            muted,
            red,
            accent_rgb,
            source,
        }
    }

    pub fn color_scheme_u32(&self) -> u32 {
        self.mode as u32
    }
}

pub fn parse_hex(input: &str) -> Option<[u8; 3]> {
    let s = input.trim().trim_start_matches('#');
    match s.len() {
        3 => {
            let r = u8::from_str_radix(&s[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&s[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&s[2..3].repeat(2), 16).ok()?;
            Some([r, g, b])
        }
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some([r, g, b])
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_omarchy_colors_toml() {
        let toml = r##"
mode = "dark"
accent = "#89b4fa"
background = "#1e1e2e"
foreground = "#cdd6f4"
lighter_background = "#313244"
muted = "#585b70"
red = "#f38ba8"
"##;
        let theme = OmarchyTheme::from_toml(&toml.parse().unwrap(), PathBuf::from("colors.toml"));
        assert_eq!(theme.mode, ColorScheme::PreferDark);
        assert!((theme.accent[0] - 0.537).abs() < 0.01);
        assert_eq!(theme.accent_rgb, [0x89, 0xb4, 0xfa]);
    }

    #[test]
    fn light_mode_from_toml() {
        let theme = OmarchyTheme::from_toml(
            &"mode = \"light\"\naccent = \"#aabbcc\"".parse().unwrap(),
            PathBuf::new(),
        );
        assert_eq!(theme.mode, ColorScheme::PreferLight);
        assert_eq!(parse_hex("#abc"), Some([0xaa, 0xbb, 0xcc]));
    }
}
