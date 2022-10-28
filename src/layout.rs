use crate::style::{
  StyledNode,
  Display
};
use crate::css::{
  CSSValue,
  CSSUnit
};

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
  Block(&'a StyledNode<'a>),
  Inline(&'a StyledNode<'a>),
  /// 匿名`block box`，用于存放多个`inline box`
  AnonymousBlock
}

/// 布局树（`layout tree`）节点
#[derive(Debug)]
pub struct LayoutBox<'a> {
  pub box_model: Box,
  pub box_type: BoxType<'a>,
  pub children: Vec<LayoutBox<'a>>
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

impl LayoutBox<'_> {
  fn new(box_type: BoxType) -> LayoutBox {
    LayoutBox {
      box_model: Box::default(),
      box_type,
      children: vec!()
    }
  }

  /// 获取`inline`节点的容器节点
  /// 
  /// 主要是判断在`block`节点内混用`inline`和`block`节点时，需要对连续的`inline`节点人为增加匿名容器
  fn get_inline_container(&mut self) -> &mut Self {
    if let BoxType::Inline(_) | BoxType::AnonymousBlock = self.box_type {
      self
    } else {
      // 居然还以用..来代替剩余结构
      if let Some(&LayoutBox { box_type: BoxType::AnonymousBlock, .. }) = self.children.last() {

      } else {
        self.children.push(LayoutBox::new(BoxType::AnonymousBlock));
      }
      self.children.last_mut().unwrap()
    }
  }

  /// 获取样式节点
  fn get_style_node<'a>(&'a self) -> &'a StyledNode<'a> {
    if let BoxType::Block(style_node) | BoxType::Inline(style_node) = self.box_type {
      &style_node
    } else {
      panic!("匿名结点没有样式")
    }
  }

  /// 计算块级元素宽度
  fn calc_block_width(&mut self, containing_block: Box) {
    let style_node = self.get_style_node();
    let auto = CSSValue::Keyword(String::from("auto"));
    let zero = CSSValue::Length(0.0, CSSUnit::Px);
    let mut width = style_node.get_val("width").unwrap_or(auto.clone());
    let mut margin_left = style_node.look_up("margin-left", "margin", &zero);
    let mut margin_right = style_node.look_up("margin-right", "margin", &zero);
    let padding_left = style_node.look_up("padding-left", "padding", &zero);
    let padding_right = style_node.look_up("padding-right", "padding", &zero);
    let border_left = style_node.look_up("border-left-width", "border-width", &zero);
    let border_right = style_node.look_up("border-right-width", "border-width", &zero);
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
  }

  /// 计算块级元素高度
  fn calc_block_height(&mut self) {
    // TODO: 块级元素排列算法
    if let Some(CSSValue::Length(height, CSSUnit::Px)) = self.get_style_node().get_val("height") {
      self.box_model.content.height = height;
    }
  }

  /// 计算块级元素子元素布局
  fn calc_block_children(&mut self) {
    let box_model = &mut self.box_model;
    for child in &mut self.children {
      // 自顶向下计算元素布局
      child.calc_layout(*box_model);
      // 自底向上计算元素高度
      box_model.content.height = box_model.content.height + child.box_model.margin_box().height;
    }
  }

  fn calc_block_layout(&mut self, containing_block: Box) {
    // 自顶向下计算宽度和起点
    self.calc_block_width(containing_block);
    self.calc_block_position(containing_block);
    self.calc_block_children();
    // 自底向上计算高度
    self.calc_block_height();
  }

  fn calc_layout(&mut self, containing_block: Box) {
    match self.box_type {
      BoxType::Block(_) => self.calc_block_layout(containing_block),
      BoxType::Inline(_) => {
        // TODO: 行内元素布局计算
      },
      BoxType::AnonymousBlock => {
        // TODO: 匿名容器布局计算
      }
    }
  }
}

/// 生成布局树结构
fn get_layout_tree_struct<'a>(style_tree: &'a StyledNode<'a>) -> LayoutBox<'a> {
  let mut root = LayoutBox::new(
    match style_tree.get_display() {
      Display::Block => BoxType::Block(style_tree),
      Display::Inline => BoxType::Inline(style_tree),
      Display::None => panic!("根节点不能设置`display: none`")
    }
  );

  for child in &style_tree.children {
    match child.get_display() {
      Display::Block => root.children.push(get_layout_tree_struct(child)),
      Display::Inline => root.get_inline_container().children.push(get_layout_tree_struct(child)),
      Display::None => {} // 跳过display为none的节点
    }
  }

  root
}

/// 从样式树生成布局树
pub fn get_layout_tree<'a>(style_tree: &'a StyledNode<'a>, mut init_box: Box) -> LayoutBox<'a> {
  init_box.content.height = 0.0;
  let mut root_box = get_layout_tree_struct(style_tree);
  root_box.calc_layout(init_box);
  root_box
}

