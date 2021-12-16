use std::collections::{
  HashMap,
  HashSet
};
use crate::css::Stylesheet;

pub type AttrMap = HashMap<String, String>;
#[derive(Debug)]
pub struct ElementData {
  pub tag_name: String,
  pub attrs: AttrMap,
}
#[derive(Debug)]
pub struct StyleData {
  tag_name: String,
  attrs: AttrMap,
  inner_text: String
}
#[derive(Debug)]
pub enum NodeType {
  Text(String),
  Element(ElementData),
  Comment(String),
  Style(StyleData),
}
#[derive(Debug)]
pub struct Node {
  pub node_type: NodeType,
  pub children: Vec<Node>,
}

#[derive(Debug)]
pub struct Document {
  pub root: Node,
  pub stylesheets: Vec<Stylesheet>
}

impl ElementData {
  /// 获取元素`id`列表
  pub fn ids(&self) -> HashSet<&str> {
    match self.attrs.get("id") {
      Some(val) => val.split(' ').collect(),
      None => HashSet::new()
    }
  }

  /// 获取元素类列表
  pub fn classes(&self) -> HashSet<&str> {
    match self.attrs.get("class") {
      Some(val) => val.split(' ').collect(),
      None => HashSet::new()
    }
  }
}

/// 创建`text`节点
pub fn text(data: String) -> Node {
  Node {
    node_type: NodeType::Text(data),
    children: vec!()
  }
}

/// 创建`element`节点
pub fn element(name: String, attrs: AttrMap, children: Vec<Node>) -> Node {
  Node {
    node_type: NodeType::Element(
      ElementData {
        tag_name: name,
        attrs,
      }
    ),
    children,
  }
}

/// 创建`comment`节点
pub fn comment(content: String) -> Node {
  Node {
    node_type: NodeType::Comment(content),
    children: vec!()
  }
}

/// 创建`style`节点
pub fn style(tag_name: String, attrs: AttrMap, inner_text: String) -> Node {
  Node {
    node_type: NodeType::Style(StyleData {
      tag_name,
      attrs,
      inner_text,
    }),
    children: vec!()
  }
}
