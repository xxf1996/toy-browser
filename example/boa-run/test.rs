use boa_engine::{
  Context,
};
use std::fs;
// use std::io::prelude;
use std::path::PathBuf;

fn main() {
  let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  file_path.push("example/boa-run/global.js");
  let global = fs::read_to_string(&file_path).unwrap();
  let mut context = Context::default();
  context.eval(&global).unwrap();
  let user_script = "window.name"; // 注入完原生代码后，用户代码就能访问各种原生API了；
  let value = context.eval(user_script).unwrap();
  println!("{:?}", value.as_string())
}

