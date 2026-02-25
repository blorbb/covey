use hex_color::HexColor;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
/// An _unmultiplied_ color value.
pub struct Color(HexColor);

impl Color {
    pub const WHITE: Self = Self(HexColor::WHITE);

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(HexColor::rgba(r, g, b, a))
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, u8::MAX)
    }

    #[inline]
    pub fn multiply_alpha(self, factor: f32) -> Self {
        debug_assert!(
            0.0 <= factor && factor.is_finite(),
            "factor should be finite, but was {factor}"
        );
        Self(self.0.with_a(((self.a() as f32) * factor) as u8))
    }

    pub fn r(&self) -> u8 {
        self.0.r
    }

    pub fn g(&self) -> u8 {
        self.0.g
    }

    pub fn b(&self) -> u8 {
        self.0.b
    }

    pub fn a(&self) -> u8 {
        self.0.a
    }

    pub fn split_rgba(&self) -> [u8; 4] {
        self.0.split_rgba().into()
    }

    pub fn split_rgb(&self) -> [u8; 3] {
        self.0.split_rgb().into()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub struct Padding {
    pub block: f32,
    pub inline: f32,
}

impl Padding {
    pub fn new(block: f32, inline: f32) -> Self {
        Self { block, inline }
    }

    pub fn splat(padding: f32) -> Self {
        Self::new(padding, padding)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct UserStyle {
    // window
    window_width: Option<f32>,
    max_window_height: Option<f32>,
    window_margin: Option<Padding>,
    window_rounding: Option<f32>,
    // main colors
    bg_color: Option<Color>,
    text_color: Option<Color>,
    weak_text_color: Option<Color>,
    // fonts
    font_size: Option<f32>,
    description_font_size: Option<f32>,
    // input
    input_height: Option<f32>,
    cursor_selection_bg: Option<Color>,
    input_list_gap: Option<f32>,
    // list
    list_item_gap: Option<f32>,
    list_selected_color: Option<Color>,
    list_hovered_color: Option<Color>,
    list_rounding: Option<f32>,
    list_padding: Option<Padding>,
}

/// Getters with defaults.
impl UserStyle {
    pub fn window_width(&self) -> f32 {
        self.window_width.unwrap_or(600.)
    }

    pub fn max_window_height(&self) -> f32 {
        self.max_window_height.unwrap_or(500.)
    }

    pub fn window_margin(&self) -> Padding {
        self.window_margin.unwrap_or(Padding::splat(12.))
    }

    pub fn window_rounding(&self) -> f32 {
        self.window_rounding.unwrap_or(8.)
    }

    pub fn bg_color(&self) -> Color {
        self.bg_color.unwrap_or(Color::rgb(25, 17, 19))
    }

    pub fn text_color(&self) -> Color {
        self.text_color
            .unwrap_or(Color(HexColor::WHITE.with_a(a(0.85))))
    }

    pub fn weak_text_color(&self) -> Color {
        self.weak_text_color
            .unwrap_or(self.text_color().multiply_alpha(0.6))
    }

    pub fn font_size(&self) -> f32 {
        self.font_size.unwrap_or(14.)
    }

    pub fn description_font_size(&self) -> f32 {
        self.description_font_size
            .unwrap_or(f32::round(self.font_size() * 0.85))
    }

    pub fn input_height(&self) -> f32 {
        self.input_height.unwrap_or(32.)
    }

    pub fn cursor_selection_bg(&self) -> Color {
        self.cursor_selection_bg.unwrap_or(Color::rgb(113, 51, 68))
    }

    pub fn input_list_gap(&self) -> f32 {
        self.input_list_gap.unwrap_or(12.)
    }

    pub fn list_item_gap(&self) -> f32 {
        self.list_item_gap.unwrap_or(8.)
    }

    pub fn list_selected_color(&self) -> Color {
        self.list_selected_color
            .unwrap_or(Color::WHITE.multiply_alpha(0.2))
    }

    pub fn list_hovered_color(&self) -> Color {
        self.list_hovered_color
            .unwrap_or(Color::WHITE.multiply_alpha(0.1))
    }

    pub fn list_rounding(&self) -> f32 {
        self.list_rounding.unwrap_or(4.)
    }

    pub fn list_padding(&self) -> Padding {
        self.list_padding.unwrap_or(Padding::new(4., 4.))
    }

    /// Computed property.
    pub fn max_list_height(&self) -> f32 {
        self.max_window_height()
            - 2. * self.window_margin().block
            - self.input_height()
            - self.input_list_gap()
    }

    pub fn inner_width(&self) -> f32 {
        self.window_width() - 2.0 * self.window_margin().inline
    }
}

fn a(a: f32) -> u8 {
    (a * u8::MAX as f32) as u8
}
