use crate::constants::*;
use crate::util::*;
use gdnative::api::*;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct App {
  config: Config,
  file: GodotString,
  view: GodotString,
  help: GodotString,
  file_dialog: GodotString,
  file_dialog_title: GodotString,
  file_filters: StringArray,
  about_dialog: GodotString,
  about_version: GodotString,
  portals_timer: GodotString,
  update_signal: GodotString,
  tabs: GodotString,
  stats: GodotString,
  search: GodotString,
}

#[methods]
impl App {
  fn new(_owner: &Node) -> Self {
    let mut filters = StringArray::new();
    filters.push(GodotString::from("SotAChatLog_*.txt; Chat Logs"));
    App {
      config: Config::new(),
      file: GodotString::from("VBox/Menu/File"),
      view: GodotString::from("VBox/Menu/View"),
      help: GodotString::from("VBox/Menu/Help"),
      file_dialog: GodotString::from("FileDialog"),
      file_dialog_title: GodotString::from("Select Chat Log Folder"),
      file_filters: filters,
      about_dialog: GodotString::from("AboutDialog"),
      about_version: GodotString::from("AboutDialog/VBox/Version"),
      portals_timer: GodotString::from("VBox/Tabs/Portals/Timer"),
      update_signal: GodotString::from("timeout"),
      tabs: GodotString::from("VBox/Tabs"),
      stats: GodotString::from("VBox/Tabs/Stats"),
      search: GodotString::from("search"),
    }
  }

  #[export]
  fn _ready(&self, owner: TRef<Node>) {
    if let Some(scene) = owner.get_tree() {
      scene.to_ref().set_auto_accept_quit(false);
    }

    // Connect the file menu and set shortcuts.
    owner.connect_to(&self.file, "id_pressed", "file_menu_select");
    if let Some(button) = owner.get_node_as::<MenuButton>(&self.file) {
      if let Some(popup) = button.get_popup() {
        let popup = popup.to_ref();
        popup.set_shortcut(QUIT_ID, GlobalConstants::KEY_Q, true);
        popup.set_shortcut(SEARCH_ID, GlobalConstants::KEY_L, true);
      } else {
        godot_print!("Unable to get popup from File");
      }
    }

    // Connect the help menu.
    owner.connect_to(&self.help, "id_pressed", "help_menu_select");

    // Connect the tabs.
    owner.connect_to(&self.tabs, "tab_changed", "tab_changed");
  }

  #[export]
  fn file_menu_select(&self, owner: TRef<Node>, id: i64) {
    match id {
      LOG_FOLDER_ID => {
        if let Some(dialog) = owner.get_node_as::<FileDialog>(&self.file_dialog) {
          dialog.set_title(self.file_dialog_title.clone());
          dialog.set_mode(FileDialog::MODE_OPEN_DIR);
          dialog.set_filters(self.file_filters.clone());
          if let Some(folder) = self.config.get_log_folder() {
            dialog.set_current_path(folder);
          }
          dialog.popup_centered(Vector2::zero());
        }
      }
      SEARCH_ID => {
        owner.method(&self.stats, &self.search, &[]);
      }
      QUIT_ID => owner.propagate_notification(MainLoop::NOTIFICATION_WM_QUIT_REQUEST),
      _ => {}
    }
  }

  #[export]
  fn help_menu_select(&self, owner: TRef<Node>, id: i64) {
    if id == ABOUT_ID {
      if let Some(dialog) = owner.get_node_as::<AcceptDialog>(&self.about_dialog) {
        if let Some(label) = owner.get_node_as::<Label>(&self.about_version) {
          label.set_text(GodotString::from(format!("v{}", env!("CARGO_PKG_VERSION"))));
        }
        dialog.popup_centered(Vector2::zero());
      }
    }
  }

  #[export]
  fn tab_changed(&self, owner: TRef<Node>, idx: i64) {
    if let Some(timer) = owner.get_node_as::<Timer>(&self.portals_timer) {
      self.enable_stat_menus(owner, idx == STATS_IDX);
      if idx == PORTALS_IDX {
        timer.start(-1.0);
        timer.emit_signal(self.update_signal.clone(), &[]);
      } else {
        timer.stop();
      }
    }
  }

  fn enable_stat_menus(&self, owner: TRef<Node>, enable: bool) {
    if let Some(menu) = owner.get_node_as::<MenuButton>(&self.file) {
      if let Some(popup) = menu.get_popup() {
        let popup = popup.to_ref();
        popup.set_item_disabled(popup.get_item_index(LOG_FOLDER_ID), !enable);
      }
    }

    if let Some(menu) = owner.get_node_as::<MenuButton>(&self.view) {
      if let Some(popup) = menu.get_popup() {
        for id in &[REFRESH_ID, RESISTS_ID, FILTER_ID, RESET_ID, SEARCH_ID] {
          let popup = popup.to_ref();
          popup.set_item_disabled(popup.get_item_index(*id), !enable);
        }
      }
    }
  }
}
