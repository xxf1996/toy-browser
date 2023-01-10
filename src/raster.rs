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
use ggez::mint::Vector2;
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

/// 文本渲染信息
#[derive(Debug)]
pub struct TextRenderInfo {
  /// 文本颜色
  color: CSSColor,
  /// 文本占据的矩形区域
  area: RectArea,
  /// 文本光栅化后的字符信息
  glyphs: Arc<Mutex<Vec<GlyphPosition>>>
}

/// 绘制命令
#[derive(Debug)]
pub enum DisplayCommand {
  /// 单纯矩形区域色块
  Rectangle(CSSColor, RectArea),
  /// 文本
  Text(TextRenderInfo)
}

/// ggez绘制状态信息
struct WindowState {
  display_commands: Arc<Mutex<Vec<DisplayCommand>>>,
  /// device pixel ratio
  dpr: f32
}

/// 光栅化输出窗口
pub struct RasterWindow {
  /// 窗口id，也是标题
  id: String,
  pub display_commands: Arc<Mutex<Vec<DisplayCommand>>>
}

impl TextRenderInfo {
  /// 将当前文本光栅化信息转为ggez image，方便绘制
  fn to_image(&self, ctx: &Context) -> graphics::Image {
    let w = self.area.width as usize;
    let h = self.area.height as usize;
    let text_layout = get_text_layout();
    let glyphs = self.glyphs.lock().unwrap();
    let pixel_num = w * h * 4;
    let mut pixels: Vec<u8> = vec![0; pixel_num];
    let font_color = self.color;

    // 逐字符填充光栅化信息
    for glyph in &*glyphs {
      let (_, bitmap) = text_layout.fonts[glyph.font_index].rasterize_config(glyph.key);
      for (idx, mask) in bitmap.iter().enumerate() {
        if glyph.width == 0 || glyph.height == 0 {
          continue;
        }
        let dx = idx % glyph.width;
        let dy = (idx as f32 / glyph.width as f32).floor() as usize;
        let x = glyph.x as usize + dx;
        let y = glyph.y as usize + dy;
        if x >= w || y >= h {
          continue;
        }
        let start_idx = (y * w + x) * 4; // NOTICE: 按行优先排列的索引
        pixels[start_idx] = font_color.r;
        pixels[start_idx + 1] = font_color.g;
        pixels[start_idx + 2] = font_color.b;
        pixels[start_idx + 3] = *mask;
      }
    }

    // NOTICE: 这里绘制的像素必须转换为浮点数[0, 1]（Rgba8UnormSrgb格式会自动将u8转为0到1的浮点数），不然会报错！
    graphics::Image::from_pixels(ctx, pixels.as_slice(), graphics::ImageFormat::Rgba8UnormSrgb, w as u32, h as u32)
  }
}

impl WindowState {
  /// 在ggez画布上绘制命令列表
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
        DisplayCommand::Text(info) => {
          // 要么跟之前类似把以前的字体光栅化信息直接写入到纹理（图像像素），要么基于ggez自带的text系统重写从字体布局开始写一遍……
          let text_image = info.to_image(ctx);
          let draw_param = graphics::DrawParam::new()
            .dest(Vector2 {
              x: info.area.x * self.dpr,
              y: info.area.y * self.dpr
            })
            .scale(Vector2 {
              x: self.dpr,
              y: self.dpr
            }); // TODO: 同理这里也要考虑dpr，不过单纯地使用scale进行放大会使字体看起来很模糊
          canvas.draw(&text_image, draw_param);
        }
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
    let mut display_list = self.display_commands.lock().unwrap();
    *display_list = get_display_list(layout_tree);
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
