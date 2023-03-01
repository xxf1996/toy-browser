use std::{path::PathBuf, fs};

use boa_engine::{Context, object::{ObjectInitializer, JsArray, JsFunction}, property::Attribute, prelude::JsObject, JsValue, JsString, JsResult, class::{Class, ClassBuilder}};
use gc::{ Trace, Finalize, GcCellRef };
use std::thread;
use std::time;

/// 模拟DOM节点结构
///
/// 相关参考：[Questions about creating DOM types · Issue #2117 · boa-dev/boa](https://github.com/boa-dev/boa/issues/2117)
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
    let mut js_node = this.to_object(context).unwrap(); // this对象
    let js_children_obj = js_node
      .get("children", context)
      .unwrap()
      .to_object(context)
      .unwrap();
    // 获取到js对象中的子级节点数组对象
    let js_children = JsArray::from_object(js_children_obj, context).unwrap();
    // 参数对象
    let js_new_child = args[0].to_object(context).unwrap();
    let mut rs_node = js_node
      .downcast_mut::<DomNode>()
      .unwrap();
    // 得到参数对应的rust结构
    let rs_new_child = js_new_child.downcast_ref::<DomNode>().unwrap();
    rs_node.children.push(rs_new_child.to_owned()); // 同步更新rust结构，否则downcast得到的值就是未更新的
    drop(rs_new_child); // 释放RefCell
    drop(rs_node); // 释放可变引用
    js_children.push(js_new_child, context).unwrap();
    // js_node.set("children", js_children, false, context).unwrap();
    let rs_node = js_node.downcast_ref::<DomNode>().unwrap();
    println!("append_child(downcast_ref struct): {:#?}", rs_node); // NOTICE: 此处downcast得到的结构时更新的，但是不知道为何通过global_object得到的全局对象里面的document值却是未更新的……
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
    Ok(node.to_object2(context))
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

  fn to_object2(&self, context: &mut Context) -> JsValue {
    // 通过构造函数返回一个js对象
    let constructor = Self::get_constructor(context);
    let node_type = JsValue::String(JsString::new(self.node_type.clone()));
    let obj = constructor.construct(&[node_type.clone()], None, context).unwrap();
    let children = JsArray::new(context);
    let prototype_obj = self.get_prototype(context);
    // let prototype = prototype_obj.unwrap();
    println!("appendChild: {:?}", prototype_obj.has_property("appendChild", context));
    // 需要手动加上原型对象
    obj.set_prototype(Some(prototype_obj));
    // FIXME: 简单的对属性设置属性描述符也无法让属性可以访问，实在是不能理解为啥？
    obj.set("node_type", node_type, false, context).unwrap();
    obj.set("children", children, false, context).unwrap();
    println!("obj is native: {}", obj.is_native_object());
    let obj_value = JsValue::from(obj);
    for child in &self.children {
      let child_value = child.to_object2(context);
      Self::append_child(&obj_value, &[child_value], context).unwrap(); // NOTICE: 调用原生方法保存rust结构更新
    }
    obj_value
  }

  /// FIXME: 将js内存结构转为rust内存结构，过程中存在大量的clone操作，感觉有点浪费内存？
  fn get_rust_data(js_obj: &JsObject, context: &mut Context) -> DomNode {
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
    rs_node.children = vec![];
    if children_len == 0 {
      return rs_node;
    }
    for idx in 0..children_len {
      let child = children
        .get(idx, context)
        .unwrap()
        .to_object(context)
        .unwrap();
      let rs_child = Self::get_rust_data(&child, context);
      rs_node.children.push(rs_child);
    }
    rs_node
  }

  /// 将js对象恢复成对应的rust结构
  fn to_origin_data<'a>(js_obj: &'a JsObject) -> GcCellRef<'a, DomNode> {
    js_obj.downcast_ref::<DomNode>().unwrap()
    // FIXME: 这里downcast_ref没有恢复在js环境中修改的值？
    // let mut rs_node = js_obj
    //   .downcast_ref::<DomNode>()
    //   .unwrap()
    //   .clone();
    // let js_children_obj = js_obj
    //   .get("children", context)
    //   .unwrap();
    // let js_children = js_children_obj.as_object().unwrap();
    // let children = JsArray::from_object(js_children.clone(), context).unwrap();
    // let children_len = children.length(context).unwrap();
    // if children_len == 0 {
    //   return rs_node;
    // }
    // for idx in 0..children_len {
    //   let child = children.get(idx, context).unwrap();
    //   let rs_child = Self::to_origin_data(child, context);
    //   rs_node.children.push(rs_child);
    // }
    // rs_node
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
  let document_object = document.to_object2(&mut context);
  let shared_str = JsValue::from("test");
  context.register_global_property("document", &document_object, Attribute::READONLY);
  context.register_global_property("ToyName", &shared_str, Attribute::WRITABLE);
  context.eval(&user_script).unwrap();
  // thread::sleep(time::Duration::new(1, 0)); // NOTICE: 原以为global对象的更新有阻塞，但是发现等待一段时间后，结果也没变化
  let global_boj = context.global_object().clone();
  let js_document = global_boj
    .get("document", &mut context)
    .unwrap()
    .to_object(&mut context)
    .unwrap();
  let document = DomNode::get_rust_data(&js_document, &mut context);
  println!("cur document: {:#?}", document);
  // 全局变量可以通过global_object获取到经过用户脚本修改后的值（本身在rust环境的值并不会自动改变！）
  println!("{}", global_boj.get("ToyName", &mut context).unwrap().as_string().unwrap().as_str());
}
