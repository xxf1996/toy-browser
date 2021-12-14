use crate::dom;
use std::collections::HashMap;

struct Parser {
  /// 源码字符串
  input: String,
  /// 当前位置（字符位移）
  pos: usize,
}

#[derive(Debug)]
struct CSSColor {
  r: u8,
  g: u8,
  b: u8,
  a: u8,
}

#[derive(Debug)]
enum CSSUnit {
  Px,
  Em,
  Rem
}

/// 值类型
#[derive(Debug)]
enum CSSValue {
  Color(CSSColor),
  Keyword(String),
  Length(f32, CSSUnit),
  Unknown(String)
}

#[derive(Debug)]
struct CSSPropValue {
  prop: String,
  value: CSSValue,
}

/// 简单选择器（即不包含选择器之间的关系组合用法）
#[derive(Debug)]
struct CSSSimpleSelector {
  /// ID选择器
  id: Option<String>,
  /// class列表
  class: Vec<String>,
  /// 标签名
  tag: Option<String>
}

#[derive(Debug)]
struct CSSRule {
  selectors: Vec<CSSSimpleSelector>,
  prop_value_set: Vec<CSSPropValue>
}

#[derive(Debug)]
struct Stylesheet {
  rules: Vec<CSSRule>
}

/// 解析`hex color`单个通道值
/// 
/// 相关链接：[How would I store hexedecimal values in a variable? - The Rust Programming Language Forum](https://users.rust-lang.org/t/how-would-i-store-hexedecimal-values-in-a-variable/45545)
fn parse_single_channel(val: &str) -> u8 {
  u8::from_str_radix(val, 16).unwrap_or(0)
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

  /// 解析标识符：字母数字且不能以数字开头
  fn parse_identifier(&mut self) -> String {
    if let '0'..='9' = self.next_char() {
      panic!("标识符不能以数字开头")
    } else {
      self.consume_while(|c| if let 'a'..='z' | 'A'..='Z' | '0'..='9' = c {
        true
      } else {
        false
      })
    }
  }

  /// 解析长度类型的值
  fn parse_value_length(&mut self) -> CSSValue {
    let num = self.consume_while(|c| if let '0'..='9' | '.' = c {
      true
    } else {
      false
    });
    let unit = self.consume_while(|c| c != ';');
    let mut css_unit = CSSUnit::Px;
    if unit == "px" {
      css_unit = CSSUnit::Px;
    } else if unit == "em" {
      css_unit = CSSUnit::Em;
    } else if unit == "rem" {
      css_unit = CSSUnit::Rem;
    }
    // 关于字符串转数字：https://stackoverflow.com/questions/27043268/convert-a-string-to-int
    CSSValue::Length(num.parse::<f32>().unwrap_or(0.0), css_unit)
  }

  /// 解析`hex color`类型的值
  fn parse_hex_color(&mut self) -> CSSValue {
    let hex = self.consume_while(|c| if let '0'..='9' | 'a'..='f' | 'A'..='F' = c {
      true
    } else {
      false
    });
    assert!(hex.len() == 6, "目前只实现6位hex color解析");
    let r = parse_single_channel(&hex[0..2]);
    let g = parse_single_channel(&hex[2..4]);
    let b = parse_single_channel(&hex[4..6]);
    CSSValue::Color(CSSColor {
      r,
      g,
      b,
      a: 255
    })
  }

  /// 解析单个`CSS`值
  fn parse_value(&mut self) -> CSSValue {
    match self.next_char() {
      '0'..='9' => self.parse_value_length(),
      '#' => {
        self.consume_char();
        self.parse_hex_color()
      },
      _ => CSSValue::Unknown(self.consume_while(|c| c != ';')),
    }
  }

  /// 解析单个`CSS`键值对
  fn parse_prop_value(&mut self) -> CSSPropValue {
    let prop = self.parse_identifier();
    assert!(self.consume_char() == ':');
    self.consume_whitespace();
    let value = self.parse_value();
    assert!(self.consume_char() == ';');
    CSSPropValue {
      prop,
      value,
    }
  }
}
