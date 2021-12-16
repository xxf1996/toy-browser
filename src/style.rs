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
  Stylesheet
};
use std::collections::HashMap;

type NodeStyle = HashMap<String, CSSValue>;

/// `style-tree`节点
#[derive(Debug)]
pub struct StyledNode<'a> {
  node: &'a Node,
  children: Vec<StyledNode<'a>>,
  /// 该节点命中的样式信息
  style: NodeStyle
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
    children: root.children.iter().map(|child| style_tree(child, stylesheets)).collect()
  }
}

/// 根据文档对象生成对应的`style tree`
pub fn get_style_tree<'a>(document: &'a Document) -> StyledNode<'a> {
  style_tree(&document.root, &document.stylesheets)
}
