use crate::menu::*;
use crate::util::*;
use gdnative::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct App {
  config: Config,
  file: NodePath,
  view: NodePath,
  help: NodePath,
  stats: NodePath,
  file_dialog: NodePath,
  file_dialog_title: GodotString,
}

#[methods]
impl App {
  fn _init(_owner: Node) -> Self {
    App {
      config: Config::new(),
      file: NodePath::from_str("Layout/Menu/File"),
      view: NodePath::from_str("Layout/Menu/View"),
      help: NodePath::from_str("Layout/Menu/Help"),
      stats: NodePath::from_str("Layout/Tabs/Stats"),
      file_dialog: NodePath::from_str("FolderDialog"),
      file_dialog_title: GodotString::from_str("Select Log Folder"),
    }
  }

  #[export]
  fn _ready(&self, owner: Node) {
    unsafe {
      let object = &owner.to_object();
      if let Some(file) = self.get_file(owner) {
        if let Some(mut popup) = file.get_popup() {
          let mut key = InputEventKey::new();
          key.set_control(true);
          key.set_scancode(GlobalConstants::KEY_Q);
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
            godot_print!("Unable to connect file_menu_select: {:?}", err);
          }
        } else {
          godot_print!("Unable to get popup from File");
        }
      }

      if let Some(help) = self.get_help(owner) {
        if let Some(mut popup) = help.get_popup() {
          if let Err(err) = popup.connect(
            GodotString::from_str("id_pressed"),
            Some(*object),
            GodotString::from_str("help_menu_select"),
            VariantArray::new(),
            0,
          ) {
            godot_print!("Unable to connect help_menu_select: {:?}", err);
          }
        } else {
          godot_print!("Unable to get popup from Help");
        }
      }

      if let Some(stats) = self.get_stats(owner) {
        let object = &stats.to_object();
        if let Some(view) = self.get_view(owner) {
          if let Some(mut popup) = view.get_popup() {
            let mut key = InputEventKey::new();
            key.set_scancode(GlobalConstants::KEY_F5);
            popup.set_item_accelerator(
              popup.get_item_index(REFRESH_ID),
              key.get_scancode_with_modifiers(),
            );

            key.set_scancode(GlobalConstants::KEY_ESCAPE);
            popup.set_item_accelerator(
              popup.get_item_index(RESET_ID),
              key.get_scancode_with_modifiers(),
            );

            key.set_control(true);
            key.set_scancode(GlobalConstants::KEY_R);
            popup.set_item_accelerator(
              popup.get_item_index(RESISTS_ID),
              key.get_scancode_with_modifiers(),
            );

            key.set_scancode(GlobalConstants::KEY_F);
            popup.set_item_accelerator(
              popup.get_item_index(FILTER_ID),
              key.get_scancode_with_modifiers(),
            );

            if let Err(err) = popup.connect(
              GodotString::from_str("id_pressed"),
              Some(*object),
              GodotString::from_str("view_menu_select"),
              VariantArray::new(),
              0,
            ) {
              godot_print!("Unable to connect view_menu_select: {:?}", err);
            }
          } else {
            godot_print!("Unable to get popup from View");
          }
        }
      }
    }
  }

  #[export]
  fn file_menu_select(&self, owner: Node, id: i64) {
    match id {
      LOG_FOLDER_ID => unsafe {
        if let Some(mut file_dialog) = self.get_file_dialog(owner) {
          if let Some(folder) = self.config.get_log_folder() {
            file_dialog.set_current_path(folder);
          }
          file_dialog.set_title(self.file_dialog_title.new_ref());
          file_dialog.popup_centered(Vector2::zero());
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
          godot_print!("Unable to cast node FolderDialog as FileDialog");
        }
        return file_dialog;
      } else {
        godot_print!("Unable to get node FolderDialog");
      }
    }
    None
  }

  fn get_view(&self, owner: Node) -> Option<MenuButton> {
    unsafe {
      if let Some(view) = owner.get_node(self.view.new_ref()) {
        let view = view.cast::<MenuButton>();
        if view.is_none() {
          godot_print!("Unable to cast node View as MenuButton");
        }
        return view;
      } else {
        godot_print!("Unable to get node View");
      }
    }
    None
  }

  fn get_help(&self, owner: Node) -> Option<MenuButton> {
    unsafe {
      if let Some(help) = owner.get_node(self.help.new_ref()) {
        let help = help.cast::<MenuButton>();
        if help.is_none() {
          godot_print!("Unable to cast node Help as MenuButton");
        }
        return help;
      } else {
        godot_print!("Unable to get node Help");
      }
    }
    None
  }

  fn get_stats(&self, owner: Node) -> Option<Node> {
    unsafe {
      let stats = owner.get_node(self.stats.new_ref());
      if stats.is_none() {
        godot_print!("Unable to get node Stats");
      }
      stats
    }
  }
}
