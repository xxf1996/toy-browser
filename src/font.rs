use fontdue::{self, layout::{Layout, CoordinateSystem}, Font};

pub struct TextLayout {
  pub layout: Layout,
  pub fonts: [Font; 1]
}

impl TextLayout {
  pub fn default() -> Self {
    let font_data = include_bytes!("../example/font/SmileySans-Oblique.otf") as &[u8];
    let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default()).unwrap();
    Self { layout: Layout::new(CoordinateSystem::PositiveYDown), fonts: [font] }
  }
}
