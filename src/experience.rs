use gdnative::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Experience {}

#[methods]
impl Experience {
  fn _init(_owner: Node) -> Self {
    Experience {}
  }

  #[export]
  fn _ready(&self, _owner: Node) {}
}
