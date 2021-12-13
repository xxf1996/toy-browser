use std::collections::HashMap;

pub type AttrMap = HashMap<String, String>;
#[derive(Debug)]
pub struct ElementData {
  tag_name: String,
  attrs: AttrMap,
}
#[derive(Debug)]
pub enum NodeType {
  Text(String),
  Element(ElementData),
  Comment(String),
}
#[derive(Debug)]
pub struct Node {
  node_type: NodeType,
  children: Vec<Node>,
}

/// 创建text节点
pub fn text(data: String) -> Node {
  Node {
    node_type: NodeType::Text(data),
    children: vec!()
  }
}

/// 创建element节点
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

/// 创建comment节点
pub fn comment(content: String) -> Node {
  Node {
    node_type: NodeType::Comment(content),
    children: vec!()
  }
}
