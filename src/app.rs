use crate::menu::*;
use crate::util::*;
use gdnative::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct App {
  config: Config,
  file: NodePath,
  help: NodePath,
  file_dialog: NodePath,
  file_dialog_title: GodotString,
}

#[methods]
impl App {
  fn _init(_owner: Node) -> Self {
    App {
      config: Config::new(),
      file: NodePath::from_str("Layout/Menu/File"),
      help: NodePath::from_str("Layout/Menu/Help"),
      file_dialog: NodePath::from_str("FolderDialog"),
      file_dialog_title: GodotString::from_str("Select Log Folder"),
    }
  }

  #[export]
  fn _ready(&self, owner: Node) {
    unsafe {
      // Connect the file menu.
      owner.connect_to(&self.file, "id_pressed", "file_menu_select");
      if let Some(button) = owner.get_node_as::<MenuButton>(&self.file) {
        if let Some(popup) = button.get_popup() {
          popup.set_shortcut(QUIT_ID, GlobalConstants::KEY_Q, true);
        } else {
          godot_print!("Unable to get popup from File");
        }
      }

      // Connect the help menu.
      owner.connect_to(&self.help, "id_pressed", "help_menu_select");
    }
  }

  #[export]
  fn file_menu_select(&self, owner: Node, id: i64) {
    match id {
      LOG_FOLDER_ID => unsafe {
        if let Some(mut dialog) = owner.get_node_as::<FileDialog>(&self.file_dialog) {
          if let Some(folder) = self.config.get_log_folder() {
            dialog.set_current_path(folder);
          }
          dialog.set_title(self.file_dialog_title.new_ref());
          dialog.popup_centered(Vector2::zero());
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

  #[export]
  fn help_menu_select(&self, _owner: Node, id: i64) {
    match id {
      ABOUT_ID => {
        godot_print!("About");
      }
      _ => {}
    }
  }
}
