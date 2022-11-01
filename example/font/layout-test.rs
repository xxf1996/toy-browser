use fontdue;
use fontdue::layout::{ Layout, CoordinateSystem, TextStyle, LayoutSettings, HorizontalAlign };

fn main() {
  let font_data = include_bytes!("./SourceHanSansCN-Regular.otf") as &[u8];
  let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default()).unwrap();
  // let (metrics, bitmap) = font.rasterize('一', 16.0);
  // println!("{:#?}", metrics);
  // println!("length {}, {:#?}", bitmap.len(), bitmap);

  let fonts = &[font];
  let mut layout = Layout::new(CoordinateSystem::PositiveYUp);

  layout.reset(&LayoutSettings {
    max_width: Some(100.0), // 最大行宽，超出则会进行换行
    horizontal_align: HorizontalAlign::Center, // 水平排列方式决定了每行文字（连在一起，AABB？）跟当前行进行对齐
    ..Default::default()
  });

  layout.append(fonts, &TextStyle::new("你好，", 20.0, 0));
  layout.append(fonts, &TextStyle::new("word!!!!", 28.0, 0));
  layout.append(fonts, &TextStyle::new("😅🤡", 28.0, 0));

  let lines = layout.lines().unwrap();

  println!("{:#?}", layout.glyphs());
  println!("height: {}", layout.height());
  println!("lines: {}", lines.len());
}