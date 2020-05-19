use gdnative::*;

#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(Self::register_signals)]
pub struct App {}

#[methods]
impl App {
  fn _init(_owner: Node) -> Self {
    App {}
  }

  #[export]
  fn _ready(&self, mut _owner: Node) {
    // unsafe {
    //   _owner.emit_signal(
    //     GodotString::from_str("log_folder_change"),
    //     &[Variant::from_str("/home/barugon")],
    //   );
    // }
  }

  // Custom signal for log folder change.
  fn register_signals(builder: &init::ClassBuilder<Self>) {
    builder.add_signal(init::Signal {
      name: "log_folder_change",
      args: &[],
    });
  }
}
