mod dom;
mod html;
mod css;
mod style;
mod layout;
mod raster;
mod font;
mod thread;
// use std::io::Read; // 使用read_to_string方法必须引入这个
// use std::fs::File;
use std::fs;
use std::io::Error;
use std::path::PathBuf;

fn painting_test() -> Result<(), Error> {
  let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  file_path.push("src/demo/text-test.html");
  let file_path_url = file_path.to_str().unwrap_or("");
  println!("{}", file_path_url);
  let content = fs::read_to_string(file_path_url).unwrap();
  // 模拟视窗
  let mut viewport = layout::Box::default();
  viewport.content.width = 1280.0;
  let mut save_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  save_path.push("result.png");
  let page_thread = thread::PageThread::new(viewport, save_path.to_str().unwrap_or("").to_string());
  page_thread.html_sender.send(content).unwrap();
  Ok(())
}

fn main() {
  // dom_test();
  // html_test();
  // css_test();
  // style_tree_test();
  // layout_tree_test();
  painting_test().unwrap();
}
