use std::any::Any;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread::{self, JoinHandle};

use crate::dom::{Document};
use crate::{html, style, layout, raster};
use crate::layout::{LayoutBox};
use crate::style::StyledNode;

enum ThreadInput<'a> {
  Html(String),
  Style(&'a Document),
  Layout(Arc<StyledNode<'a>>, layout::Box),
  Raster(&'a LayoutBox<'a>)
}

struct ThreadInfo<T> {
  sender: Sender<T>,
  receiver: Receiver<T>
}

pub struct PageThread {
  pub html_sender: Sender<String>,
  // style_sender: Sender<Document>,
  // layout_sender: Sender<(Arc<StyledNode<'a>>, layout::Box)>,
  // raster_sender: Sender<LayoutBox<'a>>,
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

impl PageThread {
  pub fn new(viewport: layout::Box, save_path: String) -> Self {
    let (html_sender, html_recevier) = mpsc::channel::<String>();
    let (style_sender, style_recevier) = mpsc::channel::<Document>();
    let (layout_sender, layout_recevier) = mpsc::channel::<(Arc<StyledNode>, layout::Box)>();
    let (raster_sender, raster_recevier) = mpsc::channel::<LayoutBox>();
    let style_local_sender = style_sender.clone();
    // let layout_local_sender = layout_sender.clone();
    let raster_local_sender = raster_sender.clone();
    let document_store: Arc<Mutex<Option<Document>>> = Arc::new(Mutex::new(None));
    let document_data = document_store.clone();
    // let save_p = save_path.clone();

    let html_thread = thread::spawn(move || {
      for msg in html_recevier {
        let document = html::parse(msg);
        style_local_sender.send(document).unwrap();
      }
    });

    // FIXME: Arc<T>不是线程安全的内存，因此不被支持send！同理，RefCell<T>也是适用于单线程的；
    // https://kaisery.github.io/trpl-zh-cn/ch16-04-extensible-concurrency-sync-and-send.html
    // [rust - Is it safe to `Send` struct containing `Arc` if strong_count is 1 and weak_count is 0? - Stack Overflow](https://stackoverflow.com/questions/58977260/is-it-safe-to-send-struct-containing-rc-if-strong-count-is-1-and-weak-count)
    let style_thread = thread::spawn(move || {
      for document in style_recevier {
        let mut document_ref = document_data.lock().unwrap();
        *document_ref = Some(document);
        if let Some(_) = &*document_ref {
          let style_tree = style::get_style_tree(document_data.clone());
          println!("{:?}", style_tree);
          layout_sender.send((style_tree, viewport));
        }
      }
    });

    let layout_thread = thread::spawn(move || {
      for (style_tree, init_box) in layout_recevier {
        let layout_tree = layout::get_layout_tree(style_tree, init_box);
        raster_local_sender.send(layout_tree).unwrap();
      }
    });

    let raster_thread = thread::spawn(move || {
      for layout_tree in raster_recevier {
        let painting_res = raster::raster(&layout_tree);
        painting_res.save(&save_path);
      }
    });

    Self {
      html_sender,
      html_thread,
      style_thread,
      layout_thread,
      raster_thread
    }
  }

  pub fn join(self) -> Result<(), Box<dyn Any + Send>> {
    self.html_thread.join()?;
    self.style_thread.join()?;
    self.layout_thread.join()?;
    self.raster_thread.join()
  }
}