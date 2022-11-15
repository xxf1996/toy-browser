use boa_engine::class::Class;
use boa_engine::{
  Context,
};
use std::fs;
use std::path::PathBuf;
use gc::{ Trace, Finalize };

#[derive(Debug, Trace, Finalize)]
struct ToyClass;

impl Class for ToyClass {
  const NAME: &'static str = "Toy";

  fn constructor(this: &boa_engine::JsValue, args: &[boa_engine::JsValue], context: &mut Context) -> boa_engine::JsResult<Self> {
    
  }

  fn init(class: &mut boa_engine::class::ClassBuilder<'_>) -> boa_engine::JsResult<()> {
    class.method(name, length, function)
  }
}


fn main() {
  let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  file_path.push("example/boa-run/global.js");
  let user_script = fs::read_to_string(&file_path).unwrap();
  let mut context = Context::default();
  context.register_global_class::<ToyClass>();
}
