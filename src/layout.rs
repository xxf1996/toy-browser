use crate::style::{
  StyledNode,
  Display
};
use crate::css::{
  CSSValue,
  CSSUnit
};

/// 四周边距
#[derive(Debug)]
struct EdgeSizes {
  top: f32,
  right: f32,
  bottom: f32,
  left: f32
}

/// 矩形区域
#[derive(Debug)]
struct RectArea {
  /// 起点x坐标
  x: f32,
  /// 起点y坐标
  y: f32,
  /// 宽度
  width: f32,
  /// 高度
  height: f32
}

/// 盒模型
#[derive(Debug)]
struct Box {
  /// `content-box`
  content: RectArea,
  padding: EdgeSizes,
  border: EdgeSizes,
  margin: EdgeSizes
}

/// 盒模型类型
#[derive(Debug)]
enum BoxType<'a> {
  Block(&'a StyledNode<'a>),
  Inline(&'a StyledNode<'a>),
  /// 匿名`block box`，用于存放多个`inline box`
  AnonymousBlock
}

/// 布局树（`layout tree`）节点
#[derive(Debug)]
pub struct LayoutBox<'a> {
  box_model: Box,
  box_type: BoxType<'a>,
  children: Vec<LayoutBox<'a>>
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
  fn default() -> RectArea {
    RectArea {
      x: 0.,
      y: 0.,
      width: 0.,
      height: 0.
    }
  }
}

impl Box {
  fn default() -> Box {
    Box {
      content: RectArea::default(),
      padding: EdgeSizes::default(),
      border: EdgeSizes::default(),
      margin: EdgeSizes::default()
    }
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

  fn get_style_node<'a>(&'a self) -> &'a StyledNode<'a> {
    if let BoxType::Block(style_node) | BoxType::Inline(style_node) = self.box_type {
      &style_node
    } else {
      panic!("匿名结点没有样式")
    }
  }

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
    ].iter().map(|val| val.to_px()).sum();

    if width != auto && total_width > containing_block.content.width {
      if margin_left == auto {
        margin_left = zero.clone();
      }
      if margin_right == auto {
        margin_right = zero.clone();
      }
    }

    // 包含块剩余宽度
    let rest_wdith = containing_block.content.width - total_width;
    
    match (width == auto, margin_left == auto, margin_right == auto) {
      (false, false, false) => {
        
      }
    }
  }
}

/// 生成布局树
pub fn get_layout_tree<'a>(style_tree: &'a StyledNode<'a>) -> LayoutBox<'a> {
  let mut root = LayoutBox::new(
    match style_tree.get_display() {
      Display::Block => BoxType::Block(style_tree),
      Display::Inline => BoxType::Inline(style_tree),
      Display::None => panic!("根节点不能设置`display: none`")
    }
  );

  for child in &style_tree.children {
    match child.get_display() {
      Display::Block => root.children.push(get_layout_tree(child)),
      Display::Inline => root.get_inline_container().children.push(get_layout_tree(child)),
      Display::None => {} // 跳过display为none的节点
    }
  }

  root
}

