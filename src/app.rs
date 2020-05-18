use gdnative::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct App {}

#[methods]
impl App {
  fn _init(_owner: Node) -> Self {
    App {}
  }

  #[export]
  unsafe fn _ready(&self, _owner: Node) {}
}
