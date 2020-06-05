// use crate::util::*;
use gdnative::*;
// use num_format::{Locale, ToFormattedString};

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Offline {}

#[methods]
impl Offline {
  fn _init(_owner: Node) -> Self {
    Offline {}
  }

  #[export]
  fn _ready(&self, _owner: Node) {}
}
