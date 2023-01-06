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
use std::time::{Duration};
use regex::Regex;
use tokio::runtime::Runtime;
use tokio::time::{self, Instant};

fn painting_test() -> Result<(), Error> {
  let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  file_path.push("src/demo/text-test.html");
  let file_path_url = file_path.to_str().unwrap_or("");
  println!("{}", file_path_url);
  let mut content = fs::read_to_string(file_path_url).unwrap();
  // 模拟视窗
  let mut viewport = layout::Box::default();
  viewport.content.width = 1280.0;
  let mut save_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  save_path.push("result.png");
  let page_thread = thread::PageThread::new(viewport, save_path.to_str().unwrap_or("").to_string());
  let content_reg = Regex::new(r"(there:)\{.+\}").unwrap(); // FIXME: regex真的不支持中文字符匹配？
  let rt = Runtime::new().unwrap();
  rt.block_on(async {
    page_thread.html_sender.send(content.clone()).unwrap();
    let start = Instant::now() + Duration::from_secs(10);
    let interval = Duration::from_secs(5);
    let mut intv = time::interval_at(start, interval);
    // TODO: 如何让定时器自动触发？循环？https://rust-book.junmajinlong.com/ch100/03_use_tokio_time.html#%E9%97%B4%E9%9A%94%E4%BB%BB%E5%8A%A1-tokiotimeinterval
    intv.tick().await;
    content = content_reg.replace(content.as_str(), format!("$1{{{}}}", 1)).to_string();
    page_thread.html_sender.send(content.clone()).unwrap();
    intv.tick().await;
    content = content_reg.replace(content.as_str(), format!("$1{{{}}}", 2)).to_string();
    page_thread.html_sender.send(content.clone()).unwrap();
    intv.tick().await;
    content = content_reg.replace(content.as_str(), format!("$1{{{}}}", 3)).to_string();
    page_thread.html_sender.send(content.clone()).unwrap();
  });
  page_thread.join().unwrap(); // TODO: 多线程性能测试
  Ok(())
}

fn main() {
  painting_test().unwrap();
}
