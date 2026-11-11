//! Shared type scale, row metrics and spacing for the native interface.
use gpui::{FontFallbacks, FontFeatures, FontWeight, Styled, px};
use std::sync::{Arc, OnceLock};

pub const SPACE_2: f32 = 2.;
pub const SPACE_4: f32 = 4.;
pub const SPACE_6: f32 = 6.;
pub const SPACE_8: f32 = 8.;
pub const SPACE_12: f32 = 12.;
pub const SPACE_16: f32 = 16.;
pub const SPACE_24: f32 = 24.;
pub const SPACE_32: f32 = 32.;
pub const SIDEBAR_WIDTH: f32 = 240.;
pub const PAGE_INSET: f32 = 24.;
pub const PAGE_TOP: f32 = 20.;
pub const CARD_RADIUS: f32 = 14.;
pub const FIELD_RADIUS: f32 = 8.;
pub const BADGE_RADIUS: f32 = 6.;
pub const TILE_SIZE: f32 = 28.;
pub const TILE_RADIUS: f32 = TILE_SIZE * 0.26;
pub const BUTTON_HEIGHT: f32 = 30.;
pub const BUTTON_PADDING: f32 = 14.;
pub const BAR_HEIGHT: f32 = 14.;
pub const SCAN_ROW_HEIGHT: f32 = 82.;
pub const SCAN_TEXT_HEIGHT: f32 = 50.;
pub const ROW_PADDING: f32 = 14.;

#[derive(Clone, Copy)]
pub enum TextRole {
    PageTitle,
    Title,
    SectionTitle,
    Headline,
    Body,
    RowTitle,
    Callout,
    Metadata,
    MetadataEmphasis,
    Micro,
    Total,
}

impl TextRole {
    pub fn line_height(self) -> f32 {
        self.metrics().1
    }

    fn metrics(self) -> (f32, f32, FontWeight) {
        match self {
            Self::PageTitle => (20., 26., FontWeight::SEMIBOLD),
            Self::Title => (26., 32., FontWeight::SEMIBOLD),
            Self::SectionTitle => (15., 20., FontWeight::SEMIBOLD),
            Self::Headline => (13., 18., FontWeight::SEMIBOLD),
            Self::Body => (13., 18., FontWeight::NORMAL),
            Self::RowTitle => (13., 18., FontWeight::MEDIUM),
            Self::Callout => (12., 16., FontWeight::NORMAL),
            Self::Metadata => (11., 14., FontWeight::NORMAL),
            Self::MetadataEmphasis => (11., 14., FontWeight::MEDIUM),
            Self::Micro => (10., 14., FontWeight::SEMIBOLD),
            Self::Total => (26., 32., FontWeight::BOLD),
        }
    }
}

fn fallback_fonts() -> FontFallbacks {
    static FONTS: OnceLock<FontFallbacks> = OnceLock::new();
    FONTS
        .get_or_init(|| {
            let fonts: &[&str] = if cfg!(windows) {
                &["Segoe UI", "Microsoft YaHei UI", "Microsoft YaHei"]
            } else if cfg!(target_os = "macos") {
                &["Helvetica Neue", "PingFang SC"]
            } else {
                &["Noto Sans", "Noto Sans CJK SC", "DejaVu Sans"]
            };
            FontFallbacks::from_fonts(fonts.iter().map(|font| (*font).into()).collect())
        })
        .clone()
}

pub trait TypographyExt: Styled {
    fn type_style(mut self, role: TextRole) -> Self {
        let (size, height, weight) = role.metrics();
        let text = self.text_style();
        text.font_family = Some("Onest".into());
        text.font_fallbacks = Some(fallback_fonts());
        self.text_size(px(size))
            .line_height(px(height))
            .font_weight(weight)
    }

    fn numeric(mut self) -> Self {
        static FEATURES: OnceLock<FontFeatures> = OnceLock::new();
        let text = self.text_style();
        text.font_family = Some(
            if cfg!(windows) {
                "Segoe UI"
            } else if cfg!(target_os = "macos") {
                ".SystemUIFont"
            } else {
                "Noto Sans"
            }
            .into(),
        );
        text.font_fallbacks = Some(fallback_fonts());
        text.font_features = Some(
            FEATURES
                .get_or_init(|| FontFeatures(Arc::new(vec![("tnum".into(), 1)])))
                .clone(),
        );
        self
    }
}

impl<T: Styled> TypographyExt for T {}

pub const MONOSPACE: &str = if cfg!(windows) {
    "Consolas"
} else if cfg!(target_os = "macos") {
    "Menlo"
} else {
    "DejaVu Sans Mono"
};
