use std::{path::PathBuf, fs, borrow::BorrowMut};

use boa_engine::{Context, object::{ObjectInitializer, JsArray, Object, ConstructorBuilder}, property::Attribute, prelude::JsObject, JsValue, JsString, syntax::lexer::Error, JsResult, class::{Class, ClassBuilder}};
use gc::{ Trace, Finalize, GcCellRef, GcCellRefMut };

#[derive(Debug, Trace, Finalize, Clone)]
struct DomNode {
  node_type: String,
  children: Vec<DomNode>
}

impl DomNode {
  fn new(node_type: String) -> Self {
    Self { node_type, children: vec![] }
  }

  fn get_val(value: &JsValue) -> Result<GcCellRef<DomNode>, ()> {
    if let Some(obj) = value.as_object() {
      if let Some(node) = obj.downcast_ref::<DomNode>() {
        return Ok(node)
      }
    }

    Err(())
  }

  // fn get_mut_val(value: &JsValue, context: &mut Context) -> GcCellRefMut<Object, DomNode> {
  //   let mut a = value.to_object(context).unwrap();
  //   a.downcast_mut::<DomNode>().unwrap()
  // }

  fn append_child(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // 虽然this和args本身变量都是不可变的，但是可以通过可变的context进行修改
    let js_node = this.to_object(context).unwrap();
    let js_children_obj = js_node.get("children", context).unwrap();
    let js_children = JsArray::from_object(js_children_obj.as_object().unwrap().clone(), context).unwrap();
    let js_new_child = args[0].to_object(context).unwrap();
    js_children.push(js_new_child, context).unwrap();
    js_node.set("children", js_children, false, context).unwrap();
    // 这里并不能通过downcast_ref找到对应的rust内存，可能是因为to_object返回的结构并非通过构造函数创建的，因此匹配不上
    let rs_node = js_node.downcast_ref::<DomNode>().unwrap();
    println!("{:#?}", rs_node.children);
    Ok(JsValue::Undefined)
  }

  fn to_object(&self, context: &mut Context) -> JsObject {
    let children = JsArray::new(context);
    for child in &self.children {
      children.push(child.to_object(context), context).unwrap();
    }
    // TODO: 可以考虑通过构造函数返回一个结构
    // let a = ConstructorBuilder::new(context, Self::constructor).build();
    // a.call(&a, args, context);
    let obj = ObjectInitializer::new(context)
      .property("type", self.node_type.clone(), Attribute::all())
      .property("children", children, Attribute::all())
      .function(Self::append_child, "appendChild", 1)
      .build();

    obj
  }
}

impl Class for DomNode {
  const NAME: &'static str = "DomNode";
  const LENGTH: usize = 0;

  fn init(_class: &mut ClassBuilder<'_>) -> JsResult<()> {
    Ok(())
  }

  fn constructor(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<Self> {
    context.throw_type_error("Illegal constructor")
  }
}

fn main() {
  let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  file_path.push("example/boa-run/object-test.js");
  let user_script = fs::read_to_string(&file_path).unwrap();
  let mut context = Context::default();
  let div = DomNode::new("div".to_string());
  let mut body = DomNode::new("body".to_string());
  let mut document = DomNode::new("document".to_string());
  body.children.push(div);
  document.children.push(body);
  let document_object = document.to_object(&mut context);
  let shared_str = JsValue::from("test");
  context.register_global_class::<DomNode>().unwrap();
  context.register_global_property("document", document_object, Attribute::READONLY);
  context.register_global_property("ToyName", shared_str.clone(), Attribute::WRITABLE);
  context.eval(&user_script).unwrap();
  // 全局变量可以通过global_object获取到经过用户脚本修改后的值（本身在rust环境的值并不会自动改变！）
  println!("{}", context.global_object().clone().get("ToyName", &mut context).unwrap().as_string().unwrap().as_str());
}
