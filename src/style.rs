use crate::dom::{
  Node,
  Document,
  ElementData,
  NodeType
};
use crate::css::{
  CSSValue,
  CSSSimpleSelector,
  Specificity,
  CSSRule,
  Stylesheet,
  parse_inline_style,
};
use std::collections::HashMap;

type NodeStyle = HashMap<String, CSSValue>;

/// `style-tree`节点
#[derive(Debug)]
pub struct StyledNode<'a> {
  pub node: &'a Node,
  pub children: Vec<StyledNode<'a>>,
  /// 该节点命中的样式信息
  pub style: NodeStyle
}

#[derive(Debug)]
pub enum Display {
  Inline,
  Block,
  None
}

impl<'a> StyledNode<'a> {
  /// 获取样式节点的某个样式属性值
  pub fn get_val(&self, name: &str) -> Option<CSSValue> {
    self.style.get(name).map(|val| val.clone())
  }

  /// 获取样式节点的`display`类型
  pub fn get_display(&self) -> Display {
    if let Some(CSSValue::Keyword(val)) = self.get_val("display") {
      match &*val {
        "block" => Display::Block,
        "none" => Display::None,
        _ => Display::Inline
      }
    } else {
      Display::Inline
    }
  }

  pub fn look_up(&self, key: &str, init_key: &str, init_val: &CSSValue) -> CSSValue {
    self
      .get_val(key)
      .unwrap_or_else(|| self
        .get_val(init_key)
        .unwrap_or_else(|| init_val.clone())
      )
  }
}

type MatchedRule<'a> = (Specificity, &'a CSSRule);

/// 判断简单选择器`selector`是否命中`element`节点
fn match_selector(element: &ElementData, selector: &CSSSimpleSelector) -> bool {
  if selector.tag.iter().any(|name| element.tag_name != *name) {
    return false;
  }

  let classes = element.classes();

  if selector.class.iter().any(|class| !classes.contains(&**class)) {
    return false;
  }

  let ids = element.ids();

  // String类型的解引用居然是str类型？
  if selector.id.iter().any(|id| !ids.contains(&**id)) {
    return false;
  }

  true
}

/// 从单个规则中匹配节点样式
fn match_rule<'a>(element: &ElementData, rule: &'a CSSRule) -> Option<MatchedRule<'a>> {
  rule.selectors
    .iter()
    .find(|selector| match_selector(element, &selector)) // 规则中只要有一个选择器命中就算命中了
    .map(|selector| (selector.get_specificity(), rule))
}

/// 从多个规则中匹配节点样式
fn match_rules<'a>(element: &ElementData, stylesheet: &'a Stylesheet) -> Vec<MatchedRule<'a>> {
  stylesheet.rules
    .iter()
    .filter_map(|rule| match_rule(element, rule))
    .collect()
}

/// 从多个样式表中匹配节点样式
fn specified_values(element: &ElementData, stylesheets: &Vec<Stylesheet>) -> NodeStyle {
  let mut style = HashMap::new();
  let mut rules = vec!();
  for stylesheet in stylesheets {
    let mut res = match_rules(element, stylesheet);
    rules.append(&mut res);
  }
  rules.sort_by(|&(a, _), &(b, _)| a.cmp(&b)); // 对命中的规则按照优先级从低到高进行排序（这样便于优先级高的进行覆盖）
  for (_, rule) in rules {
    for prop_value in &rule.prop_value_set {
      style.insert(prop_value.prop.clone(), prop_value.value.clone());
    }
  }
  if element.attrs.contains_key("style") { // 最后解析内联样式（优先级最高，目前不考虑!important）
    let empty_str = String::from("");
    let style_content = element.attrs.get("style").unwrap_or(&empty_str);
    let prop_value_set = parse_inline_style(style_content.clone());
    for prop_value in &prop_value_set {
      style.insert(prop_value.prop.clone(), prop_value.value.clone());
    }
  }
  style
}

/// 递归方法，从`DOM tree`根节点进行样式匹配，生成对应的`style tree`
fn style_tree<'a>(root: &'a Node, stylesheets: &'a Vec<Stylesheet>) -> StyledNode<'a> {
  StyledNode {
    node: root,
    style: match root.node_type {
      NodeType::Element(ref element) => specified_values(element, stylesheets),
      NodeType::Text(_) => HashMap::new(),
      _ => HashMap::new()
    },
    children: root.children
      .iter()
      .filter_map(|child| if let NodeType::Element(elem) = &child.node_type {
        if elem.tag_name == "head" {
          None // 跳过head的解析
        } else {
          Some(style_tree(child, stylesheets))
        }
      } else {
        Some(style_tree(child, stylesheets))
      })
      .collect()
  }
}

/// 根据文档对象生成对应的`style tree`
pub fn get_style_tree<'a>(document: &'a Document) -> StyledNode<'a> {
  style_tree(&document.root, &document.stylesheets)
}
