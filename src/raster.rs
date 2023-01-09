use std::sync::{Arc, Mutex};

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
use ggez::event::EventHandler;
use ggez::mint::Vector2;
use image::{
  RgbaImage,
  Rgba
};
use ggez::{
  event,
  glam::*,
  graphics::{self, Color},
  Context, GameResult,
};

static DEFAULT_FONT_COLOR: CSSColor = CSSColor {
  r: 0,
  g: 0,
  b: 0,
  a: 255
};

static TRANSPARENT: CSSColor = CSSColor {
  r: 0,
  g: 0,
  b: 0,
  a: 0
};

#[derive(Debug)]
struct TextRenderInfo {
  color: CSSColor,
  area: RectArea,
  glyphs: Arc<Mutex<Vec<GlyphPosition>>>
}

#[derive(Debug)]
pub enum DisplayCommand {
  Rectangle(CSSColor, RectArea),
  Text(TextRenderInfo)
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

struct WindowState {
  display_commands: Arc<Mutex<Vec<DisplayCommand>>>,
  /// device pixel ratio
  dpr: f32
}

pub struct RasterWindow {
  id: String,
  pub display_commands: Arc<Mutex<Vec<DisplayCommand>>>
}

impl WindowState {
  fn draw_commands(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) {
    let display_list = self.display_commands.lock().unwrap();
    for command in &*display_list {
      match command {
        DisplayCommand::Rectangle(color, rect) => {
          let mut mb = graphics::MeshBuilder::new();
          let mut ggez_rect = rect.to_ggez_rect();
          // 考虑到dpr，所以需要的矩形区域进行相应的放大，且起点也要偏移
          ggez_rect.x *= self.dpr;
          ggez_rect.y *= self.dpr;
          ggez_rect.scale(self.dpr, self.dpr);
          mb.rectangle(graphics::DrawMode::fill(), ggez_rect, color.to_ggez_color()).unwrap();
          let mesh = graphics::Mesh::from_data(ctx, mb.build());
          let draw_param = graphics::DrawParam::new();
          canvas.draw(&mesh, draw_param);
        },
        DisplayCommand::Text(_info) => {
          // TODO: 要么跟之前类似把以前的字体光栅化信息直接写入到纹理（图像像素），要么基于ggez自带的text系统重写从字体布局开始写一遍……
        },
        // _ => {
        //   // TODO: 其它绘制操作
        // }
      }
    }
  }
}

impl event::EventHandler<ggez::GameError> for WindowState {
  fn update(&mut self, _ctx: &mut Context) -> GameResult {
    Ok(())
  }

  fn draw(&mut self, ctx: &mut Context) -> GameResult {
    let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
    self.draw_commands(ctx, &mut canvas);
    canvas.finish(ctx)?;
    Ok(())
  }
}

impl RasterWindow {
  pub fn new(id: String) -> Self {
    let display_commands: Arc<Mutex<Vec<DisplayCommand>>> = Arc::new(Mutex::new(Vec::new()));
    Self { id, display_commands }
  }

  pub fn raster(&mut self, layout_tree: &LayoutBox) {
    let init_box = layout_tree.box_model.margin_box();
    let mut display_list = self.display_commands.lock().unwrap();
    *display_list = get_display_list(layout_tree);
  }
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
    let glyphs = render_info.glyphs.lock().unwrap();

    for glyph in &*glyphs {
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

  pub fn to_image(&self, ctx: &Context) -> graphics::Image {
    let mut pixels: Vec<u8> = vec![];
    // NOTICE: Image的像素排列是列优先的……因此需要从上往下再从左往左扫描像素！！！
    for y in 0..self.height {
      for x in 0..self.width {
        let pixel = self.pixels.get(y * self.width + x).unwrap_or(&TRANSPARENT);
        pixels.extend_from_slice(&pixel.to_vec());
      }
    }
    // NOTICE: 这里绘制的像素必须转换为浮点数[0, 1]（Rgba8UnormSrgb格式会自动将u8转为0到1的浮点数），不然会报错！
    graphics::Image::from_pixels(ctx, pixels.as_slice(), graphics::ImageFormat::Rgba8UnormSrgb, self.width as u32, self.height as u32)
  }

  /// 将当前渲染结果保存为图片
  pub fn save(&self, path: &str) {
    let mut img = RgbaImage::new(self.width as u32, self.height as u32);
    for x in 0..self.width {
      for y in 0..self.height {
        let pixel = self.pixels.get(y * self.width + x).unwrap_or(&TRANSPARENT);
        let color = Rgba([pixel.r, pixel.g, pixel.b, pixel.a]);
        img.put_pixel(x as u32, y as u32, color);
      }
    }

    img.save(path).unwrap();
  }
}

/// 获取布局树的`display list`（绘制命令列表）
fn get_display_list<'a>(layout_tree: &'a LayoutBox) -> Vec<DisplayCommand> {
  let mut display_list: Vec<DisplayCommand> = vec!();
  get_display_command(layout_tree, &mut display_list);
  display_list
}

/// 获取单个布局结点的`display list`
fn get_display_command<'a, 'b>(layout_box: &'a LayoutBox, display_list: &'b mut Vec<DisplayCommand>) {
  draw_border(layout_box, display_list);
  draw_background(layout_box, display_list);
  draw_content(layout_box, display_list);
  for child in &layout_box.children {
    get_display_command(child, display_list);
  }
}

/// 获取布局结点的某个样式颜色
fn get_color(layout_box: &LayoutBox, color_name: &str) -> Option<CSSColor> {
  if let BoxType::Block(style_node) | BoxType::Inline(style_node) | BoxType::AnonymousInline(_, style_node) = &layout_box.box_type {
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
  let mut draw_one_border = |name: &str, rect: RectArea| {
    let color = get_color(layout_box, name)
      .unwrap_or(get_color(layout_box, "border-color").unwrap_or(TRANSPARENT.clone()));
    if color != TRANSPARENT {
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

/// 绘制纯文本内容
fn draw_content<'a, 'b>(layout_box: &'a LayoutBox, display_list: &'b mut Vec<DisplayCommand>) {
  match layout_box.box_type {
    BoxType::AnonymousInline(..) => {
      let color = get_color(layout_box, "color").unwrap_or(DEFAULT_FONT_COLOR);
      display_list.push(DisplayCommand::Text(TextRenderInfo {
        color,
        area: layout_box.box_model.content,
        glyphs: layout_box.glyphs.clone()
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

/// 启动一个窗口，需要注意的是event::run方法**必须要在主线程**执行（因为`event loop`的限制）
/// 
/// 启动窗口后该方法会**阻塞主线程**！
pub fn start_window(window_store: Arc<Mutex<RasterWindow>>) -> GameResult {
  let window = window_store.lock().unwrap();
  let cb = ggez::ContextBuilder::new(window.id.as_str(), "xxf");
  let (mut ctx, event_loop) = cb.build().unwrap();
  let dpr = ctx.gfx.window().scale_factor() as f32;
  let state = WindowState {
    display_commands: window.display_commands.clone(),
    dpr
  };
  ctx.gfx.set_window_title(window.id.as_str());
  ctx.gfx.set_drawable_size(1280.0 * dpr, 480.0 * dpr).unwrap();
  drop(window);
  event::run(ctx, event_loop, state)
}
