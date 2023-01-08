use std::any::Any;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread::{self, JoinHandle};

use crate::dom::{Document};
use crate::{html, style, layout, raster};
use crate::layout::{LayoutBox, LayoutTree};
use crate::style::{StyledNode, StyleTree};

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
  raster_thread: JoinHandle<()>,
  pub raster_window: Arc<Mutex<raster::RasterWindow>>
}

// impl<T> ThreadInfo<T> {
//   fn new() -> Self {
//     let (sender, receiver) = mpsc::channel::<T>();
//     Self { sender, receiver }
//   }
// }

impl PageThread {
  pub fn new(viewport: layout::Box, id: String) -> Self {
    let (html_sender, html_recevier) = mpsc::channel::<String>();
    let (style_sender, style_recevier) = mpsc::channel::<Document>();
    let (layout_sender, layout_recevier) = mpsc::channel::<StyleTree>();
    let (raster_sender, raster_recevier) = mpsc::channel::<LayoutTree>();
    // let style_local_sender = style_sender.clone();
    // let raster_local_sender = raster_sender.clone();
    let document_store: Arc<Mutex<Option<Document>>> = Arc::new(Mutex::new(None));
    let document_data = document_store.clone();
    let raster_window = Arc::new(Mutex::new(raster::RasterWindow::new(id)));
    let raster_window_store = raster_window.clone();

    let html_thread = thread::spawn(move || {
      for msg in html_recevier {
        let document = html::parse(msg);
        style_sender.send(document).unwrap();
      }
    });

    // Rc<T>不是线程安全的内存，因此不被支持send！同理，RefCell<T>也是适用于单线程的；
    // https://kaisery.github.io/trpl-zh-cn/ch16-04-extensible-concurrency-sync-and-send.html
    // [rust - Is it safe to `Send` struct containing `Arc` if strong_count is 1 and weak_count is 0? - Stack Overflow](https://stackoverflow.com/questions/58977260/is-it-safe-to-send-struct-containing-rc-if-strong-count-is-1-and-weak-count)
    let style_thread = thread::spawn(move || {
      for document in style_recevier {
        let mut document_ref = document_data.lock().unwrap();
        *document_ref = Some(document);
        if document_ref.is_some() {
          let document = document_ref.take().unwrap(); // Option的take方法可以直接拿走Some数据：https://stackoverflow.com/questions/30573188/cannot-move-data-out-of-a-mutex
          let style_tree = style::StyleTree {
            document
          };
          layout_sender.send(style_tree).unwrap();
        }
      }
    });

    let layout_thread = thread::spawn(move || {
      for style_tree in layout_recevier {
        let layout_tree = LayoutTree {
          style_tree
        };
        raster_sender.send(layout_tree).unwrap();
      }
    });

    let raster_thread = thread::spawn(move || {
      for layout_tree in raster_recevier {
        let mut raster_window_ref = raster_window_store.lock().unwrap();
        raster_window_ref.raster(&layout_tree.get_layout_tree(viewport));
        drop(raster_window_ref);
      }
    });

    Self {
      html_sender,
      html_thread,
      style_thread,
      layout_thread,
      raster_thread,
      raster_window
    }
  }

  // TODO: 把进程间的数据传递改为mutex
  // pub fn new_v2(viewport: layout::Box, save_path: String) -> Self {
  //   let (html_sender, html_recevier) = mpsc::channel::<String>();
  //   let (style_sender, style_recevier) = mpsc::channel::<()>();
  //   let (layout_sender, layout_recevier) = mpsc::channel::<()>();
  //   let (raster_sender, raster_recevier) = mpsc::channel::<()>();
  //   let style_local_sender = style_sender.clone();
  //   let document_store: Arc<Mutex<Option<Document>>> = Arc::new(Mutex::new(None));
  //   let document_ref = document_store.clone();
  //   let style_tree_store: Arc<Mutex<Option<Arc<StyledNode>>>> = Arc::new(Mutex::new(None));
  //   let style_tree_ref = style_tree_store.clone();
  //   let layout_tree_store: Arc<Mutex<Option<LayoutBox>>> = Arc::new(Mutex::new(None));
  //   let layout_tree_ref = layout_tree_store.clone();

  //   let html_thread = thread::spawn(move || {
  //     for msg in html_recevier {
  //       let mut document = document_ref.lock().unwrap();
  //       *document = Some(html::parse(msg));
  //       style_local_sender.send(()).unwrap();
  //     }
  //   });

  //   let document_ref_style = document_store.clone();

  //   let style_thread = thread::spawn(move || {
  //     for _ in style_recevier {
  //       let document = document_ref_style.lock().unwrap();
  //       if let Some(data) = &*document {
  //         let mut style_tree = style_tree_ref.lock().unwrap();
  //         *style_tree = Some(style::get_style_tree(data));
  //         layout_sender.send(()).unwrap();
  //       }
  //     }
  //   });

  //   let style_tree_ref_layout = style_tree_store.clone();

  //   let layout_thread = thread::spawn(move || {
  //     for _ in layout_recevier {
  //       let style_tree = style_tree_ref_layout.lock().unwrap();
  //       if let Some(data) = &*style_tree {
  //         let mut layout_tree = layout_tree_ref.lock().unwrap();
  //         *layout_tree = Some(layout::get_layout_tree(data.clone(), viewport));
  //         raster_sender.send(()).unwrap();
  //       }
  //     }
  //   });

  //   let layout_tree_ref_raster = layout_tree_store.clone();

  //   let raster_thread = thread::spawn(move || {
  //     for _ in raster_recevier {
  //       let layout_tree = layout_tree_ref_raster.lock().unwrap();
  //       if let Some(data) = &*layout_tree {
  //         let painting_res = raster::raster(data);
  //         painting_res.save(&save_path);
  //       }
  //     }
  //   });

  //   Self {
  //     html_sender,
  //     html_thread,
  //     style_thread,
  //     layout_thread,
  //     raster_thread
  //   }
  // }

  pub fn join(self) -> Result<(), Box<dyn Any + Send>> {
    self.html_thread.join()?;
    self.style_thread.join()?;
    self.layout_thread.join()?;
    self.raster_thread.join()
  }
}