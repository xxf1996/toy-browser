use crate::dom;
use crate::css;
use std::collections::HashMap;
use std::fs;
use std::io::Error;
use std::path::PathBuf;

struct Parser {
  /// 源码字符串
  input: String,
  /// 当前位置（字符位移）
  pos: usize,
  stylesheets: Vec<css::Stylesheet>,
}

impl Parser {
  /// 返回当前位置到末尾的字符子串
  fn cur_str(&self) -> &str {
    &self.input[self.pos..]
  }

  /// 仅返回下一个字符而不移动位置
  fn next_char(&self) -> char {
    self.cur_str().chars().next().unwrap()
  }

  /// 仅返回接下来`num`个字符而不移动位置
  // fn next_chars(&self, num: usize) -> String {
  //   let mut res = String::new();
  //   let last_len = self.cur_str().len();
  //   let chars = self.cur_str().chars();
  //   assert!(num <= last_len);
  //   for i in 0..num {
  //     res.push(chars.next().unwrap_or(' '));
  //   }
  //   assert!(res.len() == num);
  //   res
  // }

  /// 判断当前字符子串是否以`s`开头
  fn starts_with(&self, s: &str) -> bool {
    self.cur_str().starts_with(s)
  }

  /// `end of file`
  fn eof(&self) -> bool {
    self.pos >= self.input.len()
  }

  /// 从当前位置消耗一个字符
  fn consume_char(&mut self) -> char {
    let mut iter = self.cur_str().char_indices();
    let (_, cur_char) = iter.next().unwrap();
    let (next_pos, _) = iter.next().unwrap_or((1, ' '));
    self.pos += next_pos;
    cur_char
  }

  /// 连续消耗字符直至`test`函数返回`false`
  fn consume_while<F>(&mut self, test: F) -> String where F: Fn(char) -> bool {
    let mut res = String::new();
    while !self.eof() && test(self.next_char()) {
      res.push(self.consume_char());
    }
    res
  }

  /// 从当前位置开始消耗连续的空格字符
  fn consume_whitespace(&mut self) {
    self.consume_while(char::is_whitespace);
  }

  /// 解析标签名，实质上就是解析连续的`字母数字`字符串
  fn parse_tag_name(&mut self) -> String {
    // 匿名函数（rust中也称为闭包）；`..=`是连续范围操作符
    self.consume_while(|c| if let 'a'..='z' | 'A'..='Z' | '0'..='9' = c {
      true
    } else {
      false
    })
  }

  /// 解析文本节点。实质上就是连续字符（但是不能包含标签）
  fn parse_text(&mut self) -> dom::Node {
    dom::text(self.consume_while(|c| c != '<'))
  }

  /// 解析属性值
  fn parse_attr_val(&mut self) -> String {
    let open_quote = self.consume_char();
    assert!(open_quote == '"' || open_quote == '\'');
    let val = self.consume_while(|c| c != open_quote);
    assert!(self.consume_char() == open_quote);
    val
  }

  /// 解析属性key
  fn parse_attr(&mut self) -> (String, String) {
    let name = self.parse_tag_name();
    assert!(self.consume_char() == '=');
    let val = self.parse_attr_val();
    (name, val)
  }

  /// 解析多个属性（实质上就是某个标签内的所有属性）
  fn parse_attrs(&mut self) -> dom::AttrMap {
    let mut attrs = HashMap::new();
    loop {
      self.consume_whitespace();
      if self.next_char() == '>' {
        break;
      }
      let (name, val) = self.parse_attr();
      attrs.insert(name, val);
    }
    attrs
  }

  /// 解析`style`内部语法
  fn parse_style(&mut self) -> String {
    let content = self.consume_while(|c| c != '<');
    self.stylesheets.push(css::parse(content.clone()));
    content
  }

  /// 解析单个标签元素（**不包含**自闭合标签）
  fn parse_element(&mut self) -> dom::Node {
    let mut res = dom::text(" ".to_string());
    assert!(self.consume_char() == '<');
    let name = self.parse_tag_name();
    let tag_name = name.clone();
    let attrs = self.parse_attrs();
    assert!(self.consume_char() == '>');
    if name == "style" {
      let source = self.parse_style();
      res = dom::style(name, attrs, source);
    } else {
      let children = self.parse_nodes();
      res = dom::element(name, attrs, children);
    }
    assert!(self.consume_char() == '<');
    assert!(self.consume_char() == '/');
    assert!(self.parse_tag_name() == tag_name);
    assert!(self.consume_char() == '>');
    res
  }

  /// 解析注释元素
  fn parse_comment(&mut self) -> dom::Node {
    // 注释开始
    assert!(self.consume_char() == '<');
    assert!(self.consume_char() == '!');
    assert!(self.consume_char() == '-');
    assert!(self.consume_char() == '-');
    let mut content = String::new();
    loop {
      if self.starts_with("-->") {
        break;
      }
      content.push(self.consume_char());
    }
    // 注释结束
    assert!(self.consume_char() == '-');
    assert!(self.consume_char() == '-');
    assert!(self.consume_char() == '>');
    dom::comment(content)
  }

  /// 解析单个节点
  fn parse_node(&mut self) -> dom::Node {
    if self.next_char() == '<' {
      if self.starts_with("<!--") { // 匹配注释开始部分
        self.parse_comment()
      } else {
        self.parse_element()
      }
    } else {
      self.parse_text()
    }
  }

  /// 解析连续的多个节点
  fn parse_nodes(&mut self) -> Vec<dom::Node> {
    let mut nodes = vec!();
    loop {
      self.consume_whitespace();
      if self.eof() || self.starts_with("</") {
        break;
      }
      nodes.push(self.parse_node());
    }
    nodes
  }
}

/// 获取浏览器内置的样式
fn get_default_stylesheet() -> Result<css::Stylesheet, Error> {
  let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  file_path.push("src");
  file_path.push("config");
  file_path.push("default.css");
  let file_path_url = file_path.to_str().unwrap_or("");
  let content = fs::read_to_string(file_path_url)?;
  let stylesheet = css::parse(content);
  Ok(stylesheet)
}

/// 解析`html`子集语法成`DOM`节点数
pub fn parse(source: String) -> dom::Document {
  let mut parser = Parser {
    pos: 0,
    input: source,
    stylesheets: vec!()
  };
  let mut nodes = parser.parse_nodes();
  let root = if nodes.len() == 1 {
    nodes.swap_remove(0)
  } else {
    dom::element(String::from("html"), HashMap::new(), nodes)
  };
  let default_stylesheet = get_default_stylesheet().unwrap_or(css::parse(String::from("")));
  parser.stylesheets.insert(0, default_stylesheet); // 保证默认样式是优先级最低的
  dom::Document {
    root,
    stylesheets: parser.stylesheets
  }
}
