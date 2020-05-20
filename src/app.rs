use crate::util::*;
use gdnative::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct App {
  config: Config,
  file: NodePath,
  _view: NodePath,
  _help: NodePath,
  file_dialog: NodePath,
}

const LOG_FOLDER_ID: i64 = 0;
const QUIT_ID: i64 = 1;

#[methods]
impl App {
  fn _init(_owner: Node) -> Self {
    App {
      config: Config::new(),
      file: NodePath::from_str("Layout/Menu/File"),
      _view: NodePath::from_str("Layout/Menu/View"),
      _help: NodePath::from_str("Layout/Menu/Help"),
      file_dialog: NodePath::from_str("FileDialog"),
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

  #[export]
  fn file_menu_select(&self, owner: Node, id: i64) {
    match id {
      LOG_FOLDER_ID => unsafe {
        if let Some(mut file_dialog) = self.get_file_dialog(owner) {
          file_dialog.set_current_path(self.config.get_log_folder());
          file_dialog.popup(Rect2::zero());
        }
      },
      QUIT_ID => unsafe {
        if let Some(mut scene) = owner.get_tree() {
          scene.quit(0);
        }
      },
      _ => {}
    }
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

  fn get_file_dialog(&self, owner: Node) -> Option<FileDialog> {
    unsafe {
      if let Some(file_dialog) = owner.get_node(self.file_dialog.new_ref()) {
        let file_dialog = file_dialog.cast::<FileDialog>();
        if file_dialog.is_none() {
          godot_print!("Unable to cast node FileDialog as FileDialog");
        }
        return file_dialog;
      } else {
        godot_print!("Unable to get node FileDialog");
      }
    }
    None
  }
}
