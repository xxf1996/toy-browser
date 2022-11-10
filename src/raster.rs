use crate::css::{
  CSSColor,
  CSSValue
};
use crate::layout::{
  RectArea,
  LayoutBox,
  BoxType,
  get_text_layout
};
use fontdue::layout::GlyphPosition;
use image::{
  RgbaImage,
  Rgba
};

static DEFAULT_FONT_COLOR: CSSColor = CSSColor {
  r: 0,
  g: 0,
  b: 0,
  a: 255
};

#[derive(Debug)]
struct TextRenderInfo<'a> {
  color: CSSColor,
  area: RectArea,
  glyphs: &'a Vec<GlyphPosition>
}

#[derive(Debug)]
enum DisplayCommand<'a> {
  Rectangle(CSSColor, RectArea),
  Text(TextRenderInfo<'a>)
}

/// 二维画布
pub struct Canvas {
  /// 宽度（像素）
  width: usize,
  /// 高度
  height: usize,
  /// 像素列表，按行排列
  pixels: Vec<CSSColor>,
}

impl Canvas {
  pub fn new(width: usize, height: usize) -> Canvas {
    let white = CSSColor {
      r: 255,
      g: 255,
      b: 255,
      a: 255
    };
    Canvas {
      width,
      height,
      pixels: std::iter::repeat(white).take(width * height).collect() // 默认填充
    }
  }

  /// 在画布中绘制一个填充颜色的矩形
  fn draw_rect(&mut self, color: CSSColor, rect: RectArea) {
    let start_x = rect.x.clamp(0.0, self.width as f32) as usize;
    let end_x = (rect.x + rect.width).clamp(0.0, self.width as f32) as usize;
    let start_y = rect.y.clamp(0.0, self.height as f32) as usize;
    let end_y = (rect.y + rect.height).clamp(0.0, self.height as f32) as usize;
    for x in start_x..end_x {
      for y in start_y..end_y {
        self.pixels[y * self.width + x] = color;
      }
    }
  }

  fn set_font_color(&mut self, x: usize, y: usize, font_color: CSSColor, font_mask: u8) {
    let index = y * self.width + x;
    let bg_color = self.pixels.get(index).unwrap_or(&CSSColor { r: 0, g: 0, b: 0, a: 0 });
    let scale = font_mask as f32 / 255.0;
    // 和底层颜色按照alpha进行混合，字体的mask color实际上就是透明度
    let single_channel = |idx: usize| (font_color[idx] as f32 * scale + bg_color[idx] as f32 * (1.0 - scale)).round() as u8;

    self.pixels[index] = CSSColor {
      r: single_channel(0),
      g: single_channel(1),
      b: single_channel(2),
      a: 255
    }
  }

  fn draw_text(&mut self, render_info: &TextRenderInfo) {
    let text_layout = get_text_layout();
    let origin_x = render_info.area.x;
    let origin_y = render_info.area.y;

    for glyph in render_info.glyphs {
      let (_, bitmap) = text_layout.fonts[glyph.font_index].rasterize_config(glyph.key);
      for (idx, mask) in bitmap.iter().enumerate() {
        if glyph.width == 0 || glyph.height == 0 {
          continue;
        }
        let dx = idx % glyph.width;
        let dy = (idx as f32 / glyph.width as f32).floor() as usize;
        let x = (origin_x + glyph.x) as usize + dx; // 将局部坐标转为基于当前inline box为起点的全局坐标
        let y = (origin_y + glyph.y) as usize + dy;
        if x >= self.width || y >= self.height {
          continue;
        }
        self.set_font_color(x, y, render_info.color, *mask);
      }
    }
  }

  /// 将当前渲染结果保存为图片
  pub fn save(&self, path: &str) {
    let mut img = RgbaImage::new(self.width as u32, self.height as u32);
    let transparent = CSSColor {
      r: 0,
      g: 0,
      b: 0,
      a: 0
    };
    for x in 0..self.width {
      for y in 0..self.height {
        let pixel = self.pixels.get(y * self.width + x).unwrap_or(&transparent);
        let color = Rgba([pixel.r, pixel.g, pixel.b, pixel.a]);
        img.put_pixel(x as u32, y as u32, color);
      }
    }

    img.save(path).unwrap();
  }
}

/// 获取布局树的`display list`（绘制命令列表）
fn get_display_list<'a>(layout_tree: &'a LayoutBox) -> Vec<DisplayCommand<'a>> {
  let mut display_list: Vec<DisplayCommand> = vec!();
  get_display_command(layout_tree, &mut display_list);
  display_list
}

/// 获取单个布局结点的`display list`
fn get_display_command<'a, 'b>(layout_box: &'a LayoutBox, display_list: &'b mut Vec<DisplayCommand<'a>>) {
  draw_border(layout_box, display_list);
  draw_background(layout_box, display_list);
  draw_content(layout_box, display_list);
  for child in &layout_box.children {
    get_display_command(child, display_list);
  }
}

/// 获取布局结点的某个样式颜色
fn get_color(layout_box: &LayoutBox, color_name: &str) -> Option<CSSColor> {
  if let BoxType::Block(style_node) | BoxType::Inline(style_node) = layout_box.box_type {
    if let Some(CSSValue::Color(color)) = style_node.get_val(color_name) {
      Some(color)
    } else {
      None
    }
  } else {
    None
  }
}

/// 绘制边框图形区域
fn draw_border(layout_box: &LayoutBox, display_list: &mut Vec<DisplayCommand>) {
  let transparent = CSSColor {
    r: 0,
    g: 0,
    b: 0,
    a: 0
  };
  let mut draw_one_border = |name: &str, rect: RectArea| {
    let color = get_color(layout_box, name)
      .unwrap_or(get_color(layout_box, "border-color").unwrap_or(transparent.clone()));
    if color != transparent {
      display_list.push(DisplayCommand::Rectangle(color, rect))
    }
  };
  let box_model = &layout_box.box_model;
  let border_box = box_model.border_box();
  draw_one_border("border-top-color", RectArea {
    x: border_box.x,
    y: border_box.y,
    width: border_box.width,
    height: box_model.border.top,
  });
  draw_one_border("border-right-color", RectArea {
    x: border_box.x + border_box.width - box_model.border.right,
    y: border_box.y,
    width: box_model.border.right,
    height: border_box.height
  });
  draw_one_border("border-bottom-color", RectArea {
    x: border_box.x,
    y: border_box.y + border_box.height - box_model.border.bottom,
    width: border_box.width,
    height: box_model.border.bottom
  });
  draw_one_border("border-left-color", RectArea {
    x: border_box.x,
    y: border_box.y,
    width: box_model.border.left,
    height: border_box.height
  });
}

/// 绘制元素背景区域（目前是`padding-box`区域）
fn draw_background(layout_box: &LayoutBox, display_list: &mut Vec<DisplayCommand>) {
  if let Some(color) = get_color(layout_box, "background-color") {
    display_list.push(DisplayCommand::Rectangle(color, layout_box.box_model.padding_box()))
  }
}

fn draw_content<'a, 'b>(layout_box: &'a LayoutBox, display_list: &'b mut Vec<DisplayCommand<'a>>) {
  match layout_box.box_type {
    BoxType::AnonymousInline(_) => {
      // TODO: 当前匿名box都丢失了样式信息
      display_list.push(DisplayCommand::Text(TextRenderInfo {
        color: DEFAULT_FONT_COLOR,
        area: layout_box.box_model.content,
        glyphs: &layout_box.glyphs
      }))
    },
    _ => {}
  }
}

/// 在指定画布上绘制命令列表
fn draw_commands(display_list: &Vec<DisplayCommand>, canvas: &mut Canvas) {
  for command in display_list {
    match command {
      DisplayCommand::Rectangle(color, rect) => {
        canvas.draw_rect(*color, *rect);
      },
      DisplayCommand::Text(info) => {
        canvas.draw_text(info);
      },
      // _ => {
      //   // TODO: 其它绘制操作
      // }
    }
  }
}

/// 对布局树进行光栅化处理
pub fn raster(layout_tree: &LayoutBox) -> Canvas {
  let init_box = layout_tree.box_model.margin_box();
  let mut canvas = Canvas::new(init_box.width as usize, init_box.height as usize);
  let display_list = get_display_list(layout_tree);
  // println!("{:#?}", layout_tree.box_model);
  draw_commands(&display_list, &mut canvas);
  canvas
}

