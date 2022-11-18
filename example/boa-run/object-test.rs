use std::{path::PathBuf, fs};

use boa_engine::{Context, object::{ObjectInitializer, JsArray, JsFunction}, property::Attribute, prelude::JsObject, JsValue, JsString, JsResult, class::{Class, ClassBuilder}};
use gc::{ Trace, Finalize, GcCellRef };
use std::marker::Sized;

/// 模拟DOM节点结构
#[derive(Debug, Trace, Finalize, Clone)]
struct DomNode {
  /// 节点类型
  node_type: String,
  /// 子级节点
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

  /// 模拟DOM节点原生的appendChild方法
  fn append_child(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // 虽然this和args本身变量都是不可变的，但是可以通过可变的context进行修改
    let js_node = this.to_object(context).unwrap(); // this对象
    let js_children_obj = js_node.get("children", context).unwrap();
    // 获取到js对象中的子级节点数组对象
    let js_children = JsArray::from_object(js_children_obj.as_object().unwrap().clone(), context).unwrap();
    // 参数对象
    let js_new_child = args[0].to_object(context).unwrap();
    // 得到参数对应的rust结构
    let rs_new_child = js_new_child.downcast_ref::<DomNode>().unwrap();
    // 从rust类型转为对应的js对象
    let child = rs_new_child.to_object(context);
    js_children.push(child, context).unwrap();
    js_node.set("children", js_children, false, context).unwrap();
    Ok(JsValue::Undefined) // js返回值
  }

  /// 获取到DomNode类型注册到js上下文中的构造函数对象
  ///
  /// 主要是boa没有暴露有关[ClassBuilder::new](https://boa-dev.github.io/boa/doc/boa_engine/class/struct.ClassBuilder.html#method.new)的API，不然使用该API会很方便得到构造函数
  fn get_constructor(context: &mut Context) -> JsFunction {
    let js_obj = context.global_object().clone();
    let constructor_obj = js_obj.get(Self::NAME, context).unwrap().to_object(context).unwrap();
    JsFunction::from_object(constructor_obj).unwrap()
  }

  /// 获取当前节点对应的原型对象（prototype）
  ///
  /// 主要是挂载一些原生方法
  fn get_prototype(&self, context: &mut Context) -> JsObject {
    let mut prototype_obj = ObjectInitializer::new(context);
    prototype_obj.function(Self::append_child, "appendChild", 1);
    if self.node_type == "document" {
      prototype_obj.function(Self::create_element, "createElement", 1);
    }
    prototype_obj.build()
  }

  /// 模拟Document类型节点的createElement方法
  fn create_element(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let node_type = args[0]
      .to_string(context)
      .unwrap()
      .to_string();
    let node = Self {
      node_type,
      children: vec![]
    };
    Ok(JsValue::Object(node.to_object(context)))
  }

  /// 将当前rust结构转为对应的js对象
  ///
  /// 通过ObjectInitializer构造的js对象不能通过downcast_ref进行结构恢复
  ///
  /// 但是将DomNode注册为全局class后，通过构造函数（即new）创建的js对象失去了对原本属性的访问及prototype的访问
  ///
  /// FIXME: 实在是不理解这是故意设计成这种特性还是bug？因此还需要在构造完js对象后手动加上属性和prototype，否则js环境下无法访问！
  ///
  /// 使用native class构造js对象的好处就是该对象会被识别成native object，**因此可以通过downcast_ref进行rust的结构恢复**
  fn to_object(&self, context: &mut Context) -> JsObject {
    let children = JsArray::new(context);
    for child in &self.children {
      children.push(child.to_object(context), context).unwrap();
    }
    // 通过构造函数返回一个js对象
    let constructor = Self::get_constructor(context);
    let node_type = JsValue::String(JsString::new(self.node_type.clone()));
    let obj = constructor.construct(&[node_type.clone()], None, context).unwrap();
    let prototype_obj = self.get_prototype(context);
    // let prototype = prototype_obj.unwrap();
    println!("appendChild: {:?}", prototype_obj.has_property("appendChild", context));
    // 需要手动加上原型对象
    obj.set_prototype(Some(prototype_obj));
    // FIXME: 简单的对属性设置属性描述符也无法让属性可以访问，实在是不能理解为啥？
    obj.set("node_type", node_type, false, context).unwrap();
    obj.set("children", children, false, context).unwrap();
    println!("obj is native: {}", obj.is_native_object());
    // obj.set_prototype(a);
    // obj.define_property_or_throw("node_type", desc, context).unwrap();
    // let obj = ObjectInitializer::new(context)
    //   .property("type", self.node_type.clone(), Attribute::all())
    //   .property("children", children, Attribute::all())
    //   // .function(Self::append_child, "appendChild", 1)
    //   .build();
    // println!("{}", obj.is_native_object());
    obj
  }

  /// 将js对象恢复成对应的rust结构
  ///
  /// FIXME: 这里js对象和rust结构应该无法共用内存？所以只能重建内存？但是为啥downcast_ref返回的又是一个带gc的指针？
  fn to_origin_data(js_value: JsValue, context: &mut Context) -> DomNode {
    let js_obj = js_value
      .as_object()
      .unwrap();
    // FIXME: 这里downcast_ref没有恢复vec的结构，为啥？
    let mut rs_node = js_obj
      .downcast_ref::<DomNode>()
      .unwrap()
      .clone();
    let js_children_obj = js_obj
      .get("children", context)
      .unwrap();
    let js_children = js_children_obj.as_object().unwrap();
    let children = JsArray::from_object(js_children.clone(), context).unwrap();
    let children_len = children.length(context).unwrap();
    if children_len == 0 {
      return rs_node;
    }
    for idx in 0..children_len {
      let child = children.get(idx, context).unwrap();
      let rs_child = Self::to_origin_data(child, context);
      rs_node.children.push(rs_child);
    }
    rs_node
  }
}

impl Class for DomNode {
  const NAME: &'static str = "DomNode";
  const LENGTH: usize = 1;
  const ATTRIBUTES: Attribute = Attribute::all();

  fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
    // let a = PropertyDescriptorBuilder::new().enumerable(true).build();
    class.method("append_child", 1, Self::append_child);
    // class.property_descriptor("node_type", a);
    Ok(())
  }

  fn constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<Self> {
    let node_type = args[0].to_string(context).unwrap().to_string();
    // context.throw_type_error("Illegal constructor")
    Ok(Self { node_type, children: vec![] })
  }
}

fn main() {
  let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  file_path.push("example/boa-run/object-test.js");
  let user_script = fs::read_to_string(&file_path).unwrap();
  let mut context = Context::default();
  context.register_global_class::<DomNode>().unwrap();
  let div = DomNode::new("div".to_string());
  let mut body = DomNode::new("body".to_string());
  let mut document = DomNode::new("document".to_string());
  body.children.push(div);
  document.children.push(body);
  let document_object = document.to_object(&mut context);
  let shared_str = JsValue::from("test");
  context.register_global_property("document", document_object, Attribute::READONLY);
  context.register_global_property("ToyName", shared_str.clone(), Attribute::WRITABLE);
  context.eval(&user_script).unwrap();
  // 全局变量可以通过global_object获取到经过用户脚本修改后的值（本身在rust环境的值并不会自动改变！）
  println!("{}", context.global_object().clone().get("ToyName", &mut context).unwrap().as_string().unwrap().as_str());
  let js_document = context
    .global_object()
    .clone()
    .get("document", &mut context)
    .unwrap();
  // let document = js_document
  //   .as_object()
  //   .unwrap()
  //   .downcast_ref::<DomNode>()
  //   .unwrap();
  let document = DomNode::to_origin_data(js_document, &mut context);
  println!("cur document: {:#?}", document)
}
