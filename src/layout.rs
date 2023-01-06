use std::sync::Arc;

use fontdue::layout::{TextStyle, GlyphPosition, LayoutSettings};

use crate::dom::NodeType;
use crate::font::TextLayout;
use crate::style::{
  StyledNode,
  Display, StyleTree
};
use crate::css::{
  CSSValue,
  CSSUnit
};

/// [Global variables? Do they exist? : rust](https://www.reddit.com/r/rust/comments/2v2h8l/global_variables_do_they_exist/)
///
/// 在rust里，限定了全局变量的声明方式，过于动态的全局变量是unsafe的
static mut TEXT_LAYOUTS: Vec<TextLayout> = vec![];

/// 四周边距
#[derive(Debug, Copy, Clone)]
pub struct EdgeSizes {
  pub top: f32,
  pub right: f32,
  pub bottom: f32,
  pub left: f32
}

/// 矩形区域
#[derive(Debug, Copy, Clone)]
pub struct RectArea {
  /// 起点x坐标
  pub x: f32,
  /// 起点y坐标
  pub y: f32,
  /// 宽度
  pub width: f32,
  /// 高度
  pub height: f32
}

/// 盒模型
#[derive(Debug, Copy, Clone)]
pub struct Box {
  /// `content-box`
  pub content: RectArea,
  pub padding: EdgeSizes,
  pub border: EdgeSizes,
  pub margin: EdgeSizes
}

/// 盒模型类型
#[derive(Debug)]
pub enum BoxType<'a> {
  Block(Arc<StyledNode<'a>>),
  Inline(Arc<StyledNode<'a>>),
  /// 匿名`block box`，用于存放多个`inline box`
  AnonymousBlock(Arc<StyledNode<'a>>),
  /// 匿名`inline box`，一般是由块级box直接包含的文字产生，样式直接继承父级
  AnonymousInline(&'a String, Arc<StyledNode<'a>>),
  /// line box
  Line
}

/// 布局树（`layout tree`）节点
#[derive(Debug)]
pub struct LayoutBox<'a> {
  pub box_model: Box,
  pub box_type: BoxType<'a>,
  pub children: Vec<LayoutBox<'a>>,
  pub glyphs: Vec<GlyphPosition>,
}

pub struct LayoutTree {
  pub style_tree: StyleTree
}

impl EdgeSizes {
  fn default() -> EdgeSizes {
    EdgeSizes {
      top: 0.,
      right: 0.,
      bottom: 0.,
      left: 0.
    }
  }
}

impl RectArea {
  /// 默认值
  fn default() -> RectArea {
    RectArea {
      x: 0.,
      y: 0.,
      width: 0.,
      height: 0.
    }
  }

  /// 在矩形区域外扩展四周`边距`，形成一个**新的矩形区域**
  fn expanded_by(self, edge: EdgeSizes) -> RectArea {
    RectArea {
      x: self.x - edge.left,
      y: self.y - edge.top,
      width: self.width + edge.left + edge.right,
      height: self.height + edge.top + edge.bottom,
    }
  }
}

impl Box {
  /// 默认值
  pub fn default() -> Box {
    Box {
      content: RectArea::default(),
      padding: EdgeSizes::default(),
      border: EdgeSizes::default(),
      margin: EdgeSizes::default()
    }
  }

  /// `padding-box`区域
  pub fn padding_box(self) -> RectArea {
    self.content.expanded_by(self.padding)
  }

  /// `border-box`区域
  pub fn border_box(self) -> RectArea {
    self.padding_box().expanded_by(self.border)
  }

  /// `margin-box`区域
  pub fn margin_box(self) -> RectArea {
    self.border_box().expanded_by(self.margin)
  }
}

impl<'a> LayoutBox<'a> {
  fn new(box_type: BoxType) -> LayoutBox {
    LayoutBox {
      box_model: Box::default(),
      box_type,
      children: vec![],
      glyphs: vec![]
    }
  }

  /// 获取`inline`节点的容器节点（这里的self就是`inline`节点的父节点）
  /// 
  /// 主要是判断在`block`节点内混用`inline`和`block`节点时，需要对连续的`inline`节点人为增加匿名容器
  fn get_inline_container(&mut self) -> &mut Self {
    // 本身如果是匿名块级box或内联box则无需新建容器
    match &self.box_type {
      BoxType::Inline(_) | BoxType::AnonymousBlock(_) => self,
      BoxType::Block(style_node) => {
        // TODO: 上一个元素如果正好是匿名块级box则无需再新建，直接共用？标准里好像没见到……
        // 按理说，如果自身是block box，且子级正好是非匿名的inline box还有必要借用匿名block box吗？
        if let Some(&LayoutBox { box_type: BoxType::AnonymousBlock(_), .. }) = self.children.last() {
          //
        } else {
          self.children.push(LayoutBox::new(BoxType::AnonymousBlock(style_node.clone())));
        }
        self.children.last_mut().unwrap() // 返回匿名块级box
      },
      _ => self // 其他的情况应该不需要处理
    }
  }

  /// 获取样式节点
  fn get_style_node(&self) -> Arc<StyledNode<'a>> {
    if let BoxType::Block(style_node) | BoxType::Inline(style_node) | BoxType::AnonymousBlock(style_node) = &self.box_type {
      style_node.clone()
    } else {
      // TODO: 其他盒模型的样式与继承
      panic!("匿名结点没有样式！{:#?}", self.box_type)
    }
  }

  /// 计算块级元素宽度
  fn calc_block_width(&mut self, containing_block: Box, is_anonymous: bool) {
    let style_node = self.get_style_node();
    let auto = CSSValue::Keyword(String::from("auto"));
    let zero = CSSValue::Length(0.0, CSSUnit::Px);
    let mut width = style_node.get_val("width").unwrap_or(auto.clone());
    let mut margin_left = if is_anonymous { zero.clone() } else { style_node.look_up("margin-left", "margin", &zero) };
    let mut margin_right = if is_anonymous { zero.clone() } else { style_node.look_up("margin-right", "margin", &zero) };
    let padding_left = if is_anonymous { zero.clone() } else { style_node.look_up("padding-left", "padding", &zero) };
    let padding_right = if is_anonymous { zero.clone() } else { style_node.look_up("padding-right", "padding", &zero) };
    let border_left = if is_anonymous { zero.clone() } else { style_node.look_up("border-left-width", "border-width", &zero) };
    let border_right = if is_anonymous { zero.clone() } else { style_node.look_up("border-right-width", "border-width", &zero) };
    let total_width: f32 = [
      &margin_left,
      &border_left,
      &padding_left,
      &width,
      &padding_right,
      &border_right,
      &margin_right
    ].iter().map(|val| val.to_px()).sum(); // 总宽度（实际上就是`margin-box`宽度）

    // 当前元素总宽度超过其包含块宽度时
    if width != auto && total_width > containing_block.content.width {
      // 首先压缩外边距宽度
      if margin_left == auto {
        margin_left = zero.clone();
      }
      if margin_right == auto {
        margin_right = zero.clone();
      }
    }

    //TODO: 包含块剩余宽度（关键是上面改变外边距的行为不会导致总宽度变化吗？）
    let rest_wdith = containing_block.content.width - total_width;

    println!("width: {}, rest: {}", total_width, rest_wdith);
    
    match (width == auto, margin_left == auto, margin_right == auto) {
      (false, false, false) => {
        // 这里填充右侧外边距的目的是当溢出的时候，通过负边距来修正，而宽度剩余时只是简单地填满剩余宽度
        margin_right = CSSValue::Length(margin_right.to_px() + rest_wdith, CSSUnit::Px);
      },
      (false, true, false) => {
        margin_left = CSSValue::Length(rest_wdith, CSSUnit::Px);
      },
      (false, false, true) => {
        margin_right = CSSValue::Length(rest_wdith, CSSUnit::Px);
      },
      (false, true, true) => {
        margin_left = CSSValue::Length(rest_wdith / 2.0, CSSUnit::Px);
        margin_right = CSSValue::Length(rest_wdith / 2.0, CSSUnit::Px);
      },
      (true, _, _) => {
        // width的auto优先级最高
        if margin_left == auto {
          margin_left = zero.clone();
        }
        if margin_right == auto {
          margin_right = zero.clone();
        }
        if rest_wdith < 0.0 {
          width = zero.clone();
          // 通过边距来修正
          margin_right = CSSValue::Length(margin_right.to_px() + rest_wdith, CSSUnit::Px);
        } else {
          width = CSSValue::Length(rest_wdith, CSSUnit::Px);
          println!("此时的width: {}", width.to_px());
        }
      }
    }

    // 更新水平方向的宽度信息
    self.box_model.content.width = width.to_px();
    self.box_model.padding.left = padding_left.to_px();
    self.box_model.padding.right = padding_right.to_px();
    self.box_model.border.left = border_left.to_px();
    self.box_model.border.right = border_right.to_px();
    self.box_model.margin.left = margin_left.to_px();
    self.box_model.margin.right = margin_right.to_px();
  }

  /// 获取盒模型的竖直方向距离信息
  /// 
  /// 因为`rust`限制了在同一作用域对同一变量同时进行可变和不可变引用
  fn get_box_vertical_info(&self) -> (f32, f32, f32, f32, f32, f32) {
    if let BoxType::AnonymousBlock(_) = self.box_type {
      (0.0, 0.0, 0.0, 0.0, 0.0, 0.0) // 匿名块级元素应该忽略样式
    } else {
      let style_node = self.get_style_node();
      let zero = CSSValue::Length(0.0, CSSUnit::Px);
      (
        style_node.look_up("margin-top", "margin", &zero).to_px(),
        style_node.look_up("margin-bottom", "margin", &zero).to_px(),
        style_node.look_up("border-top-width", "border-width", &zero).to_px(),
        style_node.look_up("border-bottom-width", "border-width", &zero).to_px(),
        style_node.look_up("padding-top", "padding", &zero).to_px(),
        style_node.look_up("padding-bottom", "padding", &zero).to_px(),
      )
    }
  }

  /// 计算块级元素位置
  fn calc_block_position(&mut self, containing_block: Box) {
    let vertical_info = self.get_box_vertical_info();
    let box_model = &mut self.box_model;
    box_model.margin.top = vertical_info.0;
    box_model.margin.bottom = vertical_info.1;
    box_model.border.top = vertical_info.2;
    box_model.border.bottom = vertical_info.3;
    box_model.padding.top = vertical_info.4;
    box_model.padding.bottom = vertical_info.5;
    // 计算当前盒模型的`content-box`起点位置；以其包含块`content-box`的起点进行相对位移
    box_model.content.x = containing_block.content.x + box_model.margin.left + box_model.border.left + box_model.padding.left;
    // 当前包含块的高度就是之前的子级元素撑开的高度，需要累加到当前元素的偏移中！
    box_model.content.y = containing_block.content.y + containing_block.content.height + box_model.margin.top + box_model.border.top + box_model.padding.top;
    println!("border box: {:#?}", box_model.border);
    println!("padding box: {:#?}", box_model.padding);
    println!("content box: {:#?}", box_model.content);
  }

  /// 计算块级元素高度
  fn calc_block_height(&mut self) {
    if let Some(CSSValue::Length(height, CSSUnit::Px)) = self.get_style_node().get_val("height") {
      self.box_model.content.height = height;
    }
  }

  /// 计算块级元素子元素布局
  fn calc_block_children(&mut self) {
    self.calc_block_line_box(); // 先计算line box，因为line box本质上改变了box tree的结构
    let box_model = &mut self.box_model;
    // 考虑到line box是动态产生的，这里应该用栈结构进行遍历
    for child in &mut self.children {
      // 自顶向下计算元素布局
      child.calc_layout(*box_model);
      // 自底向上计算元素高度
      box_model.content.height = box_model.content.height + child.box_model.margin_box().height;
    }
  }

  /// 将inline box的子级全部平展到一维（应该是深度优先遍历？）
  fn flat_inline_box<'b>(&mut self) -> Vec<LayoutBox<'a>> {
    // 这里'b的生命周期应该在'a之内？
    let mut all_children: Vec<LayoutBox<'_>> = vec![];
    while self.children.len() > 0 {
      let mut child = self.children.remove(0);
      match child.box_type {
        BoxType::AnonymousInline(..) => {
          all_children.push(child)
        },
        BoxType::Inline(_) => {
          let children = child.flat_inline_box();
          all_children.extend(children)
        },
        _ => {}
      }
    }
    all_children
  }

  /// 获取当前`line box`的剩余宽度
  fn get_line_rest_width(&self) -> f32 {
    if let BoxType::Line = self.box_type {
      self.box_model.content.width - self.children.iter().map(|child| child.box_model.content.width).sum::<f32>()
    } else {
      0.0
    }
  }

  /// 计算block box内部的line box结构
  ///
  /// 这里顺便计算了line box内部文本（匿名inline box）的宽度，高度和起始位置
  fn calc_block_line_box(&mut self) {
    if self.children.len() == 0 {
      return;
    }
    let mut all_children: Vec<LayoutBox<'_>> = vec![];
    while self.children.len() > 0 {
      let mut cur_child = self.children.remove(0);
      match cur_child.box_type {
        BoxType::Block(_) | BoxType::AnonymousBlock(_) | BoxType::AnonymousInline(..) => {
          all_children.push(cur_child)
        },
        BoxType::Inline(_) => {
          // 这里相当于把inline box及其子级全部提到当前container box中了，平展后方便进行line box的计算
          all_children.extend(cur_child.flat_inline_box())
        },
        _ => {} // 初始box tree不会产生line box，所以不需要考虑
      }
    }
    let mut line_and_children: Vec<LayoutBox<'_>> = vec![];
    while all_children.len() > 0 {
      let mut cur_child = all_children.remove(0);
      match cur_child.box_type {
        BoxType::Block(_) | BoxType::AnonymousBlock(_) => {
          line_and_children.push(cur_child)
        },
        BoxType::AnonymousInline(content, _) => {
          let (w, h) = cur_child.calc_text_layout(content);
          println!("文本宽高: {w}, {h}; {content}");
          let text_layout = get_text_layout();
          cur_child.box_model.content.width = w;
          cur_child.box_model.content.height = h; // 设置行高
          cur_child.glyphs = text_layout.layout.glyphs().clone(); // TODO: 不知道这里能不能引用，主要是担心clear操作会清空
          let mut last_line: Option<&mut LayoutBox> = None;

          for child in line_and_children.iter_mut() {
            if let BoxType::Line = child.box_type {
              last_line = Some(child);
            }
          }

          if let None = last_line {
            let mut new_line = LayoutBox::new(BoxType::Line);
            new_line.box_model.content.width = self.box_model.content.width;
            line_and_children.push(new_line);
            last_line = line_and_children.last_mut();
          }

          let mut last_line_box = last_line.unwrap();
          let rest_width = last_line_box.get_line_rest_width();

          if rest_width >= w {
            println!("剩余宽度: {rest_width}");
            cur_child.box_model.content.x = last_line_box.box_model.content.width - rest_width; // 水平排列
            last_line_box.children.push(cur_child);
          } else { // line box剩余宽度不够时则新加一行（目前不考虑单行文本换行的情况）
            let mut new_line = LayoutBox::new(BoxType::Line);
            new_line.box_model.content.width = self.box_model.content.width;
            line_and_children.push(new_line);
            last_line = line_and_children.last_mut();
            last_line_box = last_line.unwrap();
            cur_child.box_model.content.x = 0.0;
            last_line_box.children.push(cur_child);
          }
        },
        _ => {} // 这里理论上不存在不包含文字的line box了
      }
    }

    self.children = line_and_children;
  }

  fn calc_block_layout(&mut self, containing_block: Box, is_anonymous: bool) {
    // 自顶向下计算宽度和起点
    self.calc_block_width(containing_block, is_anonymous);
    self.calc_block_position(containing_block);
    self.calc_block_children();
    // 自底向上计算高度
    self.calc_block_height();
  }

  fn calc_inline_children(&mut self, containing_block: Box) {
    let box_model = &mut self.box_model;
    for child in &mut self.children {
      child.calc_layout(containing_block)
    }
  }

  fn calc_inline_width(&mut self, containing_block: Box) {
    // TODO: 在哪里给line box重新分配现有的inline box？
    self.calc_inline_children(containing_block);
  }

  fn calc_inline_layout(&mut self, containing_block: Box) {
    // 头大
  }

  /// 计算单行文本的宽高信息
  fn calc_text_layout(&self, text: &String) -> (f32, f32) {
    let text_layout = get_text_layout();
    // text_layout.layout.clear();
    text_layout.layout.reset(&LayoutSettings {
      max_width: Some(10000.0), // 暂时不考虑换行
      ..Default::default()
    });
    text_layout.layout.append(&text_layout.fonts, &TextStyle::new(text.as_str(), 16.0, 0));
    // TODO: 除了超出宽度的自动换行，还有换行符可以直接触发换行，因此当文字中有换行符就不可控了
    let last_text = text_layout.layout.glyphs().last().unwrap();
    // 文字的起始位置取决于最近的一个line box；
    (last_text.x + (last_text.width as f32), text_layout.layout.height())
  }

  /// 计算line box的布局信息
  fn calc_line_box_layout(&mut self, containing_block: Box) {
    let max_h = self.children.iter().map(|child| child.box_model.content.height).max_by(|a, b| a.total_cmp(b)).unwrap();
    self.box_model.content.x = containing_block.content.x;
    self.box_model.content.y = containing_block.content.y + containing_block.content.height; // 竖直位置取决于当前包含块高度
    self.box_model.content.height = max_h; // 高度取决于当前包含的最高的inline box
    println!("line box: {:#?}", self.box_model.content);
    // 同时修正line box下所有子级的位置
    for child in self.children.iter_mut() {
      child.box_model.content.x += self.box_model.content.x;
      child.box_model.content.y += self.box_model.content.y;
    }
  }

  fn calc_layout(&mut self, containing_block: Box) {
    // 这里的包含块有可能是匿名块级box，实际上计算百分比属性时不应该用匿名块级box作为包含块

    // 经过line box的重新组织后，这里应该不再会出现inline/匿名inline的情况了
    match self.box_type {
      BoxType::Block(_) => self.calc_block_layout(containing_block, false),
      // TODO: line box怎么确定？line box只由IFC产生，那么应该都是在inline box内部？
      // 根据测试(https://codepen.io/xxf1996/pen/oNyLWLd)，同一个line box可能包含多个不同inline box的内容；因此line box确实只能存在block box内？
      BoxType::AnonymousBlock(_) => {
        // TODO: 匿名容器布局计算
        println!("AnonymousBlock");
        self.calc_block_layout(containing_block, true) // TODO: 匿名block不应该再计算padding/border/margin及一些样式，不然就重复了
      },
      BoxType::Line => {
        self.calc_line_box_layout(containing_block)
      },
      _ => {}
    }
  }
}

/// 生成布局树结构（实际上是构建box tree）
fn get_layout_tree_struct<'a>(style_tree: Arc<StyledNode<'a>>) -> LayoutBox<'a> {
  let mut root = LayoutBox::new(
    match style_tree.get_display() {
      Display::Block => BoxType::Block(style_tree.clone()),
      Display::Inline => {
        if let NodeType::Text(content) = &style_tree.node.node_type {
          BoxType::AnonymousInline(&content, style_tree.clone())
        } else {
          BoxType::Inline(style_tree.clone())
        }
      },
      Display::None => panic!("根节点不能设置`display: none`")
    }
  );

  let children = style_tree.children.lock().unwrap();

  for child in children.iter() {
    match child.get_display() {
      Display::Block => root.children.push(get_layout_tree_struct(child.clone())),
      Display::Inline => root.get_inline_container().children.push(get_layout_tree_struct(child.clone())),
      Display::None => {} // 跳过display为none的节点
    }
  }

  drop(children);

  root
}

pub fn get_text_layout<'a>() -> &'a mut TextLayout {
  unsafe {
    if TEXT_LAYOUTS.len() == 0 {
      panic!("文字布局还未加载成功！")
    }
    TEXT_LAYOUTS.get_mut(0).unwrap()
  }
}

impl LayoutTree {
  /// 从样式树生成布局树
  pub fn get_layout_tree<'a>(&'a self, mut init_box: Box) -> LayoutBox<'a> {
    let style_tree = self.style_tree.get_style_tree();
    unsafe {
      // 初始化文字布局模块
      if TEXT_LAYOUTS.len() == 0 {
        TEXT_LAYOUTS.push(TextLayout::default())
      }
    }
    init_box.content.height = 0.0;
    let mut root_box = get_layout_tree_struct(style_tree);
    root_box.calc_layout(init_box);
    root_box
  }
}

