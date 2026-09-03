//! Discover and install a system CJK font for Workbench text.
//!
//! Does not vendor font files. Does not treat one absolute Windows path as
//! architectural authority. Windows is the R1 UAT target; other desktops
//! fall back to common system families, then to egui's default Latin fonts.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::OnceLock;

use ab_glyph::{Font, FontRef};
use egui::{FontData, FontDefinitions, FontFamily};

const CJK_PROBE: [char; 3] = ['骨', '中', '一'];

static INSTALL_NOTE: OnceLock<String> = OnceLock::new();

pub fn install_note() -> Option<&'static str> {
    INSTALL_NOTE.get().map(String::as_str)
}

/// Install a discovered CJK face as the first proportional fallback.
/// Never panics if no usable font exists.
pub fn install_cjk_fonts(ctx: &egui::Context) -> bool {
    match discover_cjk_font() {
        Some(loaded) => {
            let mut fonts = FontDefinitions::default();
            let key = "rigforge_cjk".to_string();
            fonts
                .font_data
                .insert(key.clone(), Arc::new(loaded.data));
            if let Some(proportional) = fonts.families.get_mut(&FontFamily::Proportional) {
                proportional.insert(0, key.clone());
            }
            if let Some(monospace) = fonts.families.get_mut(&FontFamily::Monospace) {
                monospace.push(key);
            }
            ctx.set_fonts(fonts);
            let _ = INSTALL_NOTE.set(format!(
                "已加载系统字体：{}",
                loaded.source_path.display()
            ));
            true
        }
        None => {
            let _ = INSTALL_NOTE.set(
                "未找到系统中文字体。界面文字可能显示为方框。可通过环境变量 RIGFORGE_CJK_FONT 指定字体文件。"
                    .into(),
            );
            false
        }
    }
}

pub struct LoadedCjkFont {
    pub source_path: PathBuf,
    pub data: FontData,
}

pub fn discover_cjk_font() -> Option<LoadedCjkFont> {
    for path in candidate_font_paths() {
        if let Some(data) = load_usable_cjk_font(&path) {
            return Some(LoadedCjkFont {
                source_path: path,
                data,
            });
        }
    }
    None
}

fn candidate_font_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(explicit) = std::env::var("RIGFORGE_CJK_FONT") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            paths.push(PathBuf::from(trimmed));
        }
    }
    paths.extend(platform_candidates());
    paths
}

fn platform_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    #[cfg(windows)]
    {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
        let fonts = Path::new(&windir).join("Fonts");
        for name in [
            "msyh.ttc",
            "msyhl.ttc",
            "msyh.ttf",
            "msjh.ttc",
            "simhei.ttf",
            "simsun.ttc",
            "Deng.ttf",
            "deng.ttf",
            "mingliu.ttc",
            "msyhbd.ttc",
        ] {
            paths.push(fonts.join(name));
        }
    }
    #[cfg(target_os = "macos")]
    {
        for path in [
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/STHeiti Medium.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
            "/System/Library/Fonts/Supplemental/Songti.ttc",
        ] {
            paths.push(PathBuf::from(path));
        }
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        for path in [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.otf",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            "/usr/share/fonts/truetype/arphic/uming.ttc",
        ] {
            paths.push(PathBuf::from(path));
        }
    }
    paths
}

fn load_usable_cjk_font(path: &Path) -> Option<FontData> {
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() < 12 {
        return None;
    }
    let max_face = if is_ttc(&bytes) { 8 } else { 1 };
    for index in 0..max_face {
        if let Some(data) = font_data_if_cjk(&bytes, index) {
            return Some(data);
        }
    }
    None
}

fn is_ttc(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && &bytes[0..4] == b"ttcf"
}

fn font_data_if_cjk(bytes: &[u8], index: u32) -> Option<FontData> {
    let font = FontRef::try_from_slice_and_index(bytes, index).ok()?;
    if !has_cjk_glyph(&font) {
        return None;
    }
    let mut data = FontData::from_owned(bytes.to_vec());
    data.index = index;
    Some(data)
}

fn has_cjk_glyph(font: &FontRef<'_>) -> bool {
    CJK_PROBE.iter().any(|ch| font.glyph_id(*ch).0 != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cjk_discovery_does_not_panic() {
        let _ = discover_cjk_font();
    }

    #[cfg(windows)]
    #[test]
    fn windows_system_cjk_font_is_discoverable() {
        let loaded = discover_cjk_font().expect("Windows UAT hosts should have a CJK system font");
        assert!(
            loaded.source_path.exists(),
            "discovered font missing: {}",
            loaded.source_path.display()
        );
        let ctx = egui::Context::default();
        assert!(install_cjk_fonts(&ctx));
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            ctx.fonts(|fonts| {
                assert!(
                    fonts.has_glyph(&egui::FontId::proportional(16.0), '骨'),
                    "installed font must render CJK glyphs"
                );
            });
        });
    }
}
