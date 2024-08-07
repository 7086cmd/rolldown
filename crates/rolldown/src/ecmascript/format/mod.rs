use rolldown_sourcemap::{ConcatSource, RawSource};

pub mod app;
pub mod cjs;
pub mod esm;
pub mod iife;

pub trait AddRawString {
  fn add_raw(&mut self, content: String);
  fn add_optional_raw(&mut self, content: Option<String>);
}

impl AddRawString for ConcatSource {
  fn add_raw(&mut self, content: String) {
    self.add_source(Box::new(RawSource::new(content)));
  }

  fn add_optional_raw(&mut self, content: Option<String>) {
    if let Some(content) = content {
      self.add_raw(content);
    }
  }
}
