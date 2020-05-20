use gdnative::*;

#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(Self::register_signals)]
pub struct App {
  file: NodePath,
  _view: NodePath,
  _help: NodePath,
}

const LOG_FOLDER_ID: i64 = 0;
const QUIT_ID: i64 = 1;

#[methods]
impl App {
  fn _init(_owner: Node) -> Self {
    App {
      file: NodePath::from_str("Layout/Menu/File"),
      _view: NodePath::from_str("Layout/Menu/View"),
      _help: NodePath::from_str("Layout/Menu/Help"),
    }
  }

  #[export]
  fn _ready(&self, owner: Node) {
    unsafe {
      let mut key = InputEventKey::new();
      key.set_control(true);
      key.set_scancode(GlobalConstants::KEY_Q);

      let object = &owner.to_object();
      if let Some(file) = self.get_file(owner) {
        if let Some(mut popup) = file.get_popup() {
          popup.set_item_accelerator(
            popup.get_item_index(QUIT_ID),
            key.get_scancode_with_modifiers(),
          );
          if let Err(err) = popup.connect(
            GodotString::from_str("id_pressed"),
            Some(*object),
            GodotString::from_str("file_menu_select"),
            VariantArray::new(),
            0,
          ) {
            godot_print!("Unable to connect set_log_folder: {:?}", err);
          }
        } else {
          godot_print!("Unable to get popup from File");
        }
      }
    }
  }

  // Custom signal for log folder change.
  fn register_signals(builder: &init::ClassBuilder<Self>) {
    builder.add_signal(init::Signal {
      name: "log_folder_change",
      args: &[],
    });
  }

  fn get_file(&self, owner: Node) -> Option<MenuButton> {
    unsafe {
      if let Some(file) = owner.get_node(self.file.new_ref()) {
        let file = file.cast::<MenuButton>();
        if file.is_none() {
          godot_print!("Unable to cast node File as MenuButton");
        }
        return file;
      } else {
        godot_print!("Unable to get node File");
      }
    }
    None
  }

  #[export]
  fn file_menu_select(&self, owner: Node, id: i64) {
    match id {
      LOG_FOLDER_ID => {
        godot_print!("Log Folder pressed");
        // unsafe {
        //   _owner.emit_signal(
        //     GodotString::from_str("log_folder_change"),
        //     &[Variant::from_str("/home/barugon")],
        //   );
        // }
      }
      QUIT_ID => unsafe {
        if let Some(mut scene) = owner.get_tree() {
          scene.quit(0);
        }
      },
      _ => {}
    }
  }
}
