use std::rc::Rc;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread::{self, JoinHandle};

use crate::dom::{Document};
use crate::{html, style, layout, raster};
use crate::layout::{Box, LayoutBox};
use crate::style::StyledNode;

enum ThreadInput<'a> {
  Html(String),
  Style(&'a Document),
  Layout(Rc<StyledNode<'a>>, Box),
  Raster(&'a LayoutBox<'a>)
}

struct ThreadInfo<T> {
  sender: Sender<T>,
  receiver: Receiver<T>
}

struct PageThread<'a> {
  html_sender: Sender<String>,
  style_sender: Sender<&'a Document>,
  layout_sender: Sender<(&'a Rc<StyledNode<'a>>, Box)>,
  raster_sender: Sender<&'a LayoutBox<'a>>,
  html_thread: JoinHandle<()>,
  style_thread: JoinHandle<()>,
  layout_thread: JoinHandle<()>,
  raster_thread: JoinHandle<()>
}

// impl<T> ThreadInfo<T> {
//   fn new() -> Self {
//     let (sender, receiver) = mpsc::channel::<T>();
//     Self { sender, receiver }
//   }
// }

impl<'a> PageThread<'a> {
  pub fn new(viewport: Box, save_path: &str) -> Self {
    let (html_sender, html_recevier) = mpsc::channel::<String>();
    let (style_sender, style_recevier) = mpsc::channel::<&Document>();
    let (layout_sender, layout_recevier) = mpsc::channel::<(&Rc<StyledNode>, Box)>();
    let (raster_sender, raster_recevier) = mpsc::channel::<&LayoutBox>();

    let html_thread = thread::spawn(move || {
      for msg in html_recevier {
        let document = html::parse(msg);
        style_sender.send(&document);
      }
    });

    // FIXME: Rc<T>不是线程安全的内存，因此不被支持send！
    // https://kaisery.github.io/trpl-zh-cn/ch16-04-extensible-concurrency-sync-and-send.html
    // [rust - Is it safe to `Send` struct containing `Rc` if strong_count is 1 and weak_count is 0? - Stack Overflow](https://stackoverflow.com/questions/58977260/is-it-safe-to-send-struct-containing-rc-if-strong-count-is-1-and-weak-count)
    let style_thread = thread::spawn(move || {
      for document in style_recevier {
        let style_tree = style::get_style_tree(document);
        layout_sender.send((&style_tree, viewport));
      }
    });

    let layout_thread = thread::spawn(move || {
      for (style_tree, init_box) in layout_recevier {
        let layout_tree = layout::get_layout_tree(*style_tree, init_box);
        raster_sender.send(&layout_tree);
      }
    });

    let raster_thread = thread::spawn(move || {
      for layout_tree in raster_recevier {
        let painting_res = raster::raster(layout_tree);
        painting_res.save(save_path);
      }
    });


    Self {
      html_sender,
      style_sender,
      layout_sender,
      raster_sender,
      html_thread,
      style_thread,
      layout_thread,
      raster_thread
    }
  }
}

fn some() {
  todo!()
}

fn test() {
  let t1 = thread::spawn(some);
  let (send, recv) = mpsc::channel();
  send.send("cfff")
}
