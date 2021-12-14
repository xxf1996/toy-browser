// 本来想用trait来复用一部分方法，但是trait不允许访问self的属性，用get方法实在是有点蠢；如果要改变属性值还要另外写对应的set方法；
// https://stackoverflow.com/questions/28219730/is-it-possible-to-access-struct-fields-from-within-a-trait
pub trait CommonParser {
  fn get_input(&self) -> String;
  fn get_pos(&self) -> usize;

  /// 返回当前位置到末尾的字符子串
  fn cur_str(&self) -> &str {
    &self.get_input()[self.get_pos()..]
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
}
