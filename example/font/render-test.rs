use std::path::PathBuf;

use fontdue;
use fontdue::layout::{ Layout, CoordinateSystem, TextStyle, LayoutSettings, HorizontalAlign, WrapStyle };
use image::{ RgbaImage, Rgba };

fn get_font_color(bg_color: Rgba<u8>, base_color: Rgba<u8>, mask_color: u8) -> Rgba<u8> {
  let scale = mask_color as f32 / 255.0;
  // 和底层颜色按照alpha进行混合，字体的mask color实际上就是透明度
  let single_channel = |idx: usize| (base_color[idx] as f32 * scale + bg_color[idx] as f32 * (1.0 - scale)).round() as u8;
  Rgba([single_channel(0), single_channel(1), single_channel(2), 255])
}

fn main() {
  let font_data = include_bytes!("./SmileySans-Oblique.otf") as &[u8];
  let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default()).unwrap();

  let fonts = &[font];
  let mut layout = Layout::new(CoordinateSystem::PositiveYDown);

  layout.reset(&LayoutSettings {
    max_width: Some(200.0), // 最大行宽，超出则会进行换行
    wrap_style: WrapStyle::Letter, // 换行方式，默认会按词进行换行（所以是怎么判断连续字符是一个词的？）
    horizontal_align: HorizontalAlign::Left, // 水平排列方式决定了每行文字（连在一起，AABB？）跟当前行进行对齐
    ..Default::default()
  });

  layout.append(fonts, &TextStyle::new("How to round a number up or down in Rust?", 16.0, 0));
  layout.append(fonts, &TextStyle::new("玩具相机、微缩景观、流行色彩、高调、低调、动态色调、柔焦、局部色彩（红色/橙色/黄色/绿色/蓝色/紫色）", 20.0, 0));
  layout.append(fonts, &TextStyle::new("😅🤡", 28.0, 0));
  layout.append(fonts, &TextStyle::new("電子先幕+メカニカル+電子シャッター", 14.0, 0));

  let lines = layout.lines().unwrap();

  // println!("{:#?}", layout.glyphs());
  println!("height: {}", layout.height());
  println!("lines: {}", lines.len());

  let width = 200;
  let height = layout.height().ceil() as u32;
  let mut img = RgbaImage::new(width, height);
  let bg_color = Rgba::<u8>([255, 255, 255, 255]);
  let font_color = Rgba::<u8>([131, 163, 0, 255]);
  for x in 0..width {
    for y in 0..height {
      img.put_pixel(x, y, bg_color)
    }
  }

  for glyph in layout.glyphs() {
    let (_, bitmap) = fonts[glyph.font_index].rasterize_config(glyph.key);
    for (idx, color) in bitmap.iter().enumerate() {
      let dx = (idx % glyph.width) as u32;
      let dy = (idx as f32 / glyph.width as f32).floor() as u32;
      let x = glyph.x as u32;
      let y = glyph.y as u32;
      img.put_pixel(x + dx, y + dy, get_font_color(bg_color, font_color, *color));
    }
    // println!("字符: {}, {:?}", glyph.parent, bitmap.len());
  }

  let mut save_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  save_path.push("font-test.png");

  img.save(save_path.to_str().unwrap_or("")).unwrap();
}