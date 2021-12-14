mod dom;
mod html;
mod css;
use dom::{
  Node,
  text,
  element,
  comment,
};
use std::collections::HashMap;
// use std::io::Read; // 使用read_to_string方法必须引入这个
// use std::fs::File;
use std::fs;
use std::io::Error;
use std::path::PathBuf;

fn dom_test() {
  let mut children: Vec<Node> = vec!();
  children.push(element(String::from("p"), HashMap::new(), vec!()));
  children.push(comment(String::from("<!-- swe -->")));
  children.push(text(String::from("content")));
  let document = element(String::from("div"), HashMap::new(), children);
  println!("{:#?}", document);
}

fn html_test() -> Result<(), Error> {
  let source = String::from("<html><body xxx=\"123\">hello parser</body></html>");
  let res = html::parse(source);
  println!("{:#?}", res);
  // let project_root = env!("CARGO_MANIFEST_DIR");
  // println!("{}", project_root);
  // CARGO_MANIFEST_DIR是内置的环境：项目根目录路径
  let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  file_path.push("src");
  file_path.push("source.html");
  let file_path_url = file_path.to_str().unwrap_or("");
  println!("{}", file_path_url);
  let content = fs::read_to_string(file_path_url)?;
  println!("{:#?}", html::parse(content));
  Ok(())
}

fn main() {
  // dom_test();
  html_test();
}
