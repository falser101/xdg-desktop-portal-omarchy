use crate::paths::{config_home, current_theme_dir};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

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
    pub selection: [u8; 3],
    pub icon_theme: String,
    pub gtk_theme: String,
    pub font_family: String,
    pub font_pt: f32,
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
            selection: [0x45, 0x47, 0x5a],
            icon_theme: "hicolor".into(),
            gtk_theme: "Adwaita-dark".into(),
            font_family: "Inter".into(),
            font_pt: 10.0,
            source: PathBuf::new(),
        }
    }
}

struct Cache {
    stamp: u64,
    theme: OmarchyTheme,
}

static CACHE: Mutex<Option<Cache>> = Mutex::new(None);

impl OmarchyTheme {
    pub fn load() -> Self {
        let stamp = appearance_stamp();
        if let Ok(guard) = CACHE.lock() {
            if let Some(hit) = guard.as_ref() {
                if hit.stamp == stamp {
                    return hit.theme.clone();
                }
            }
        }
        let theme = Self::load_fresh();
        if let Ok(mut guard) = CACHE.lock() {
            *guard = Some(Cache {
                stamp,
                theme: theme.clone(),
            });
        }
        theme
    }

    fn load_fresh() -> Self {
        let dir = current_theme_dir();
        let mut theme = Self::load_from(&dir.join("colors.toml")).unwrap_or_else(|_| Self {
            source: dir.join("colors.toml"),
            ..Self::default()
        });
        theme.icon_theme = read_icon_theme();
        let gtk = read_gtk_settings();
        if !gtk.theme.is_empty() {
            theme.gtk_theme = gtk.theme;
        }
        if !gtk.font_family.is_empty() {
            theme.font_family = gtk.font_family;
        }
        if gtk.font_pt > 0.0 {
            theme.font_pt = gtk.font_pt;
        }
        theme
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
        let selection = parse_hex(&get("selection").unwrap_or_else(|| "#45475a".into()))
            .unwrap_or(panel);

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
            selection,
            source,
            ..Self::default()
        }
    }

    pub fn color_scheme_u32(&self) -> u32 {
        self.mode as u32
    }

    /// Map GTK UI pt (stock Omarchy is 10) onto the dialog type scale.
    pub fn type_scale(&self) -> f32 {
        (self.font_pt / 10.0).clamp(0.85, 1.6)
    }
}

fn appearance_stamp() -> u64 {
    let mut stamp = 0u64;
    for path in appearance_files() {
        let ns = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        stamp = stamp.wrapping_mul(0x100000001b3).wrapping_add(ns);
    }
    stamp
}

fn appearance_files() -> Vec<PathBuf> {
    let dir = current_theme_dir();
    let cfg = config_home();
    vec![
        dir.join("colors.toml"),
        dir.join("icons.theme"),
        cfg.join("gtk-4.0/settings.ini"),
        cfg.join("gtk-3.0/settings.ini"),
    ]
}

fn read_icon_theme() -> String {
    let from_omarchy = std::fs::read_to_string(current_theme_dir().join("icons.theme"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    if let Some(name) = from_omarchy {
        return name;
    }
    gtk_ini_value("gtk-icon-theme-name").unwrap_or_else(|| "hicolor".into())
}

struct GtkSettings {
    theme: String,
    font_family: String,
    font_pt: f32,
}

fn read_gtk_settings() -> GtkSettings {
    let mut out = GtkSettings {
        theme: String::new(),
        font_family: String::new(),
        font_pt: 0.0,
    };
    if let Some(name) = gtk_ini_value("gtk-theme-name") {
        out.theme = name;
    }
    if let Some(raw) = gtk_ini_value("gtk-font-name") {
        let (family, pt) = parse_gtk_font(&raw);
        out.font_family = family;
        out.font_pt = pt;
    }
    out
}

fn gtk_ini_value(key: &str) -> Option<String> {
    let prefix = format!("{key}=");
    for rel in ["gtk-4.0/settings.ini", "gtk-3.0/settings.ini"] {
        let Ok(text) = std::fs::read_to_string(config_home().join(rel)) else {
            continue;
        };
        for line in text.lines() {
            if let Some(value) = line.trim().strip_prefix(&prefix) {
                let value = value.trim().trim_matches('"').trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// GTK stores `Family,  10` or `Family 10`.
pub fn parse_gtk_font(raw: &str) -> (String, f32) {
    let raw = raw.trim();
    if let Some((name, size)) = raw.rsplit_once(',') {
        let pt = size.trim().parse::<f32>().unwrap_or(10.0);
        return (name.trim().to_string(), pt);
    }
    if let Some(idx) = raw.rfind(|c: char| c.is_ascii_digit()) {
        let start = raw[..=idx]
            .rfind(|c: char| !c.is_ascii_digit() && c != '.')
            .map(|i| i + 1)
            .unwrap_or(0);
        if let Ok(pt) = raw[start..=idx].parse::<f32>() {
            let name = raw[..start].trim().trim_end_matches(',').trim();
            if !name.is_empty() {
                return (name.to_string(), pt);
            }
        }
    }
    (raw.to_string(), 10.0)
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
selection = "#45475a"
"##;
        let theme = OmarchyTheme::from_toml(&toml.parse().unwrap(), PathBuf::from("colors.toml"));
        assert_eq!(theme.mode, ColorScheme::PreferDark);
        assert!((theme.accent[0] - 0.537).abs() < 0.01);
        assert_eq!(theme.accent_rgb, [0x89, 0xb4, 0xfa]);
        assert_eq!(theme.selection, [0x45, 0x47, 0x5a]);
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

    #[test]
    fn gtk_font_name_with_comma() {
        assert_eq!(parse_gtk_font("Inter,  10"), ("Inter".into(), 10.0));
        assert_eq!(parse_gtk_font("Noto Sans 11"), ("Noto Sans".into(), 11.0));
        assert_eq!(parse_gtk_font("JetBrainsMono Nerd Font, 12"), ("JetBrainsMono Nerd Font".into(), 12.0));
    }

    #[test]
    fn type_scale_tracks_gtk_pt() {
        let mut t = OmarchyTheme::default();
        t.font_pt = 10.0;
        assert!((t.type_scale() - 1.0).abs() < f32::EPSILON);
        t.font_pt = 12.0;
        assert!((t.type_scale() - 1.2).abs() < 0.001);
    }

    #[test]
    fn load_picks_up_live_omarchy_and_gtk() {
        let t = OmarchyTheme::load();
        assert!(!t.font_family.is_empty());
        assert!(t.font_pt > 0.0);
        assert!(!t.icon_theme.is_empty());
    }
}
