mod node;
use node::{
  Node,
  text,
  element,
  comment,
};
use std::collections::HashMap;

fn main() {
  let mut children: Vec<Node> = vec!();
  children.push(element(String::from("p"), HashMap::new(), vec!()));
  children.push(comment(String::from("// swe")));
  children.push(text(String::from("content")));
  let document = element(String::from("div"), HashMap::new(), children);
  println!("{:#?}", document);
}
