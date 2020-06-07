use crate::constants::*;
use crate::util::*;
use gdnative::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct App {
  config: Config,
  file: NodePath,
  view: NodePath,
  help: NodePath,
  file_dialog: NodePath,
  file_dialog_title: GodotString,
  file_filters: StringArray,
  about_dialog: NodePath,
  about_version: NodePath,
  portals_timer: NodePath,
  update_signal: GodotString,
  tabs: NodePath,
}

#[methods]
impl App {
  fn _init(_owner: Node) -> Self {
    let mut filters = StringArray::new();
    filters.push(&GodotString::from_str("SotAChatLog_*.txt; Chat Logs"));
    App {
      config: Config::new(),
      file: NodePath::from_str("VBox/Menu/File"),
      view: NodePath::from_str("VBox/Menu/View"),
      help: NodePath::from_str("VBox/Menu/Help"),
      file_dialog: NodePath::from_str("FileDialog"),
      file_dialog_title: GodotString::from_str("Select Chat Log Folder"),
      file_filters: filters,
      about_dialog: NodePath::from_str("AboutDialog"),
      about_version: NodePath::from_str("AboutDialog/VBox/Version"),
      portals_timer: NodePath::from_str("VBox/Tabs/Portals/Timer"),
      tabs: NodePath::from_str("VBox/Tabs"),
      update_signal: GodotString::from_str("timeout"),
    }
  }

  #[export]
  fn _ready(&self, owner: Node) {
    unsafe {
      if let Some(mut scene) = owner.get_tree() {
        scene.set_auto_accept_quit(false);
      }

      // Connect the file menu and set shortcuts.
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

      // Connect the tabs.
      owner.connect_to(&self.tabs, "tab_changed", "tab_changed");
    }
  }

  #[export]
  fn file_menu_select(&self, mut owner: Node, id: i64) {
    match id {
      LOG_FOLDER_ID => unsafe {
        if let Some(mut dialog) = owner.get_node_as::<FileDialog>(&self.file_dialog) {
          dialog.set_title(self.file_dialog_title.new_ref());
          dialog.set_mode(FileDialog::MODE_OPEN_DIR);
          dialog.set_filters(self.file_filters.new_ref());
          if let Some(folder) = self.config.get_log_folder() {
            dialog.set_current_path(folder);
          }
          dialog.popup_centered(Vector2::zero());
        }
      },
      QUIT_ID => unsafe {
        owner.propagate_notification(MainLoop::NOTIFICATION_WM_QUIT_REQUEST);
      },
      _ => {}
    }
  }

  #[export]
  fn help_menu_select(&self, owner: Node, id: i64) {
    match id {
      ABOUT_ID => {
        if let Some(mut dialog) = owner.get_node_as::<AcceptDialog>(&self.about_dialog) {
          unsafe {
            if let Some(mut label) = owner.get_node_as::<Label>(&self.about_version) {
              label.set_text(GodotString::from_str(&format!(
                "v{}",
                env!("CARGO_PKG_VERSION")
              )));
            }
            dialog.popup_centered(Vector2::zero());
          }
        }
      }
      _ => {}
    }
  }

  #[export]
  fn tab_changed(&self, owner: Node, idx: i64) {
    if let Some(mut timer) = owner.get_node_as::<Timer>(&self.portals_timer) {
      unsafe {
        self.enable_stat_menus(owner, idx == STATS_IDX);
        if idx == PORTALS_IDX {
          timer.start(-1.0);
          timer.emit_signal(self.update_signal.new_ref(), &[]);
        } else {
          timer.stop();
        }
      }
    }
  }

  fn enable_stat_menus(&self, owner: Node, enable: bool) {
    unsafe {
      if let Some(menu) = owner.get_node_as::<MenuButton>(&self.file) {
        if let Some(mut popup) = menu.get_popup() {
          popup.set_item_disabled(popup.get_item_index(LOG_FOLDER_ID), !enable);
        }
      }

      if let Some(menu) = owner.get_node_as::<MenuButton>(&self.view) {
        if let Some(mut popup) = menu.get_popup() {
          for id in &[REFRESH_ID, RESISTS_ID, FILTER_ID, RESET_ID] {
            popup.set_item_disabled(popup.get_item_index(*id), !enable);
          }
        }
      }
    }
  }
}
