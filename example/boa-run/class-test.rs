use boa_engine::class::{Class, ClassBuilder};
use boa_engine::property::{Attribute, PropertyDescriptor};
use boa_engine::{
  Context, JsResult, JsValue, JsString,
};
use std::fs;
use std::path::PathBuf;
use gc::{ Trace, Finalize };

#[derive(Debug, Trace, Finalize)]
struct ToyClass {
  version: f32,
  name: String
}

impl ToyClass {
  fn hello(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let js_obj = this.to_object(context).unwrap();
    // downcast_ref 可以将jsValue转为对应的rust类型结构，前提是这个rust类型被注册到原生类型中？
    let rs_obj = js_obj.downcast_ref::<ToyClass>().unwrap();
    println!("this: {:#?}", rs_obj);
    if !args[0].is_string() {
      return Err(JsValue::Null);
    }
    let name = args[0].to_string(context).unwrap();
    println!("hello arg: {name}"); // 绑定了的方法可以在用户代码调用时触发宿主环境的代码逻辑
    Ok(JsValue::String(JsString::new(format!("hello, {name}"))))
  }

  fn new(name: String) -> Self {
    Self { version: 1.0, name }
  }
}

impl Class for ToyClass {
  const NAME: &'static str = "Toy"; // 这里就是类名
  const LENGTH: usize = 1; // 构造函数参数长度

  fn constructor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<Self> {
    let name = if args[0].is_string() {
      args[0].to_string(context).unwrap().to_string()
    } else {
      String::from("")
    };
    Ok(Self::new(name))
  }

  fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
    // 这里的length参数指的是函数参数个数
    class.method("hello", 1, ToyClass::hello);
    Ok(())
  }
}


fn main() {
  let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  file_path.push("example/boa-run/class-test.js");
  let user_script = fs::read_to_string(&file_path).unwrap();
  let mut context = Context::default();
  context.register_global_class::<ToyClass>().unwrap(); // 这里注册的class创建出来的对象都不能访问属性？
  context.register_global_property("ToyWindow", "Toy Window", Attribute::all());
  context.eval(&user_script).unwrap();
}
