use crate::constants::*;
use crate::util::*;
use gdnative::*;
use std::{cell::RefCell, collections::HashMap};

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Stats {
  config: Config,
  data: RefCell<LogData>,
  view: NodePath,
  avatars: NodePath,
  dates: NodePath,
  notes: NodePath,
  tree: NodePath,
  status: NodePath,
  file_dialog: NodePath,
  filter_dialog: NodePath,
  filter_edit: NodePath,
  notes_dialog: NodePath,
  notes_edit: NodePath,
}

enum StatOpts<'a> {
  None,
  Resists,
  Filter(&'a str),
}

#[methods]
impl Stats {
  fn _init(_owner: Node) -> Self {
    let config = Config::new();
    let folder = if let Some(folder) = config.get_log_folder() {
      folder
    } else {
      GodotString::new()
    };
    Stats {
      config: config,
      data: RefCell::new(LogData::new(folder)),
      view: NodePath::from("/root/App/VBox/Menu/View"),
      avatars: NodePath::from("Tools/Avatars"),
      dates: NodePath::from("Tools/Dates"),
      notes: NodePath::from("Tools/Notes"),
      tree: NodePath::from("Panel/Tree"),
      status: NodePath::from("Status"),
      file_dialog: NodePath::from("/root/App/FileDialog"),
      filter_dialog: NodePath::from("/root/App/FilterDialog"),
      filter_edit: NodePath::from("/root/App/FilterDialog/VBox/FilterEdit"),
      notes_dialog: NodePath::from("/root/App/NotesDialog"),
      notes_edit: NodePath::from("/root/App/NotesDialog/VBox/NotesEdit"),
    }
  }

  #[export]
  fn _ready(&self, owner: Node) {
    unsafe {
      // Connect the view menu and set shortcuts.
      owner.connect_to(&self.view, "id_pressed", "view_menu_select");
      if let Some(button) = owner.get_node_as::<MenuButton>(&self.view) {
        if let Some(popup) = button.get_popup() {
          popup.set_shortcut(REFRESH_ID, GlobalConstants::KEY_F5, false);
          popup.set_shortcut(RESISTS_ID, GlobalConstants::KEY_R, true);
          popup.set_shortcut(FILTER_ID, GlobalConstants::KEY_F, true);
          popup.set_shortcut(RESET_ID, GlobalConstants::KEY_ESCAPE, false);
        } else {
          godot_print!("Unable to get popup from View");
        }
      }

      // Connect the avatars button.
      owner.connect_to(&self.avatars, "item_selected", "avatar_changed");

      // Connect the dates button.
      owner.connect_to(&self.dates, "item_selected", "date_changed");

      // Connect the notes button.
      owner.connect_to(&self.notes, "pressed", "notes_clicked");

      // Connect the notes dialog.
      owner.connect_to(&self.notes_dialog, "confirmed", "notes_changed");

      // Connect the log folder dialog.
      owner.connect_to(&self.file_dialog, "dir_selected", "log_folder_changed");

      // Connect the filter dialog.
      owner.connect_to(&self.filter_dialog, "confirmed", "filter_changed");
      if let Some(mut dialog) = owner.get_node_as::<ConfirmationDialog>(&self.filter_dialog) {
        dialog.register_text_enter(owner.get_node(self.filter_edit.new_ref()));
      }

      // Set some stats tree properties.
      if let Some(mut tree) = owner.get_node_as::<Tree>(&self.tree) {
        tree.set_column_expand(0, true);
        tree.set_column_min_width(0, 3);
        // tree.set_column_title(0, GodotString::from("Name"));
        // tree.set_column_title(1, GodotString::from("Value"));
        // tree.set_column_titles_visible(true);
      }
    }
    self.populate_avatars(owner);
  }

  #[export]
  fn view_menu_select(&self, owner: Node, id: i64) {
    match id {
      REFRESH_ID => self.populate_avatars(owner),
      RESISTS_ID => {
        if let Some(avatar) = self.get_current_avatar(owner) {
          if let Some(ts) = self.get_current_date(owner) {
            self.populate_stats(
              owner,
              Some(avatar.to_utf8().as_str()),
              Some(ts),
              StatOpts::Resists,
            );
          }
        }
      }
      FILTER_ID => {
        if let Some(mut dialog) = owner.get_node_as::<ConfirmationDialog>(&self.filter_dialog) {
          if let Some(mut edit) = owner.get_node_as::<LineEdit>(&self.filter_edit) {
            unsafe {
              edit.set_text(GodotString::new());
              dialog.popup_centered(Vector2::zero());
              edit.grab_focus();
            }
          }
        }
      }
      RESET_ID => {
        if let Some(avatar) = self.get_current_avatar(owner) {
          if let Some(ts) = self.get_current_date(owner) {
            self.populate_stats(
              owner,
              Some(avatar.to_utf8().as_str()),
              Some(ts),
              StatOpts::None,
            );
          }
        }
      }
      _ => {}
    }
  }

  #[export]
  fn avatar_changed(&self, owner: Node, item: i64) {
    if let Some(button) = owner.get_node_as::<OptionButton>(&self.avatars) {
      unsafe {
        let avatar = button.get_item_text(item);
        self.config.set_avatar(Some(avatar.new_ref()));

        if !avatar.is_empty() {
          self.populate_dates(owner, Some(avatar.to_utf8().as_str()));
          return;
        }
      }
    }
    self.populate_dates(owner, None);
  }

  #[export]
  fn date_changed(&self, owner: Node, item: i64) {
    if let Some(avatar) = self.get_current_avatar(owner) {
      if let Some(button) = owner.get_node_as::<OptionButton>(&self.dates) {
        unsafe {
          let ts = button.get_item_id(item);
          if ts != 0 {
            self.populate_stats(
              owner,
              Some(avatar.to_utf8().as_str()),
              Some(ts),
              StatOpts::None,
            );
            return;
          }
        }
      }
    }
    self.populate_stats(owner, None, None, StatOpts::None);
  }

  #[export]
  fn log_folder_changed(&self, owner: Node, folder: GodotString) {
    *self.data.borrow_mut() = LogData::new(folder.new_ref());
    self.config.set_log_folder(Some(folder));
    self.populate_avatars(owner);
  }

  #[export]
  fn filter_changed(&self, owner: Node) {
    if let Some(edit) = owner.get_node_as::<LineEdit>(&self.filter_edit) {
      let text = unsafe { edit.get_text() };
      if !text.is_empty() {
        if let Some(avatar) = self.get_current_avatar(owner) {
          if let Some(ts) = self.get_current_date(owner) {
            self.populate_stats(
              owner,
              Some(avatar.to_utf8().as_str()),
              Some(ts),
              StatOpts::Filter(text.to_utf8().as_str()),
            );
          }
        }
      }
    }
  }

  #[export]
  fn notes_clicked(&self, owner: Node) {
    if let Some(mut dialog) = owner.get_node_as::<ConfirmationDialog>(&self.notes_dialog) {
      if let Some(mut edit) = owner.get_node_as::<TextEdit>(&self.notes_edit) {
        if let Some(avatar) = self.get_current_avatar(owner) {
          unsafe {
            let title = GodotString::from(format!("Notes for {}", avatar.to_utf8().as_str()));
            let text = if let Some(text) = self.config.get_notes(avatar) {
              text
            } else {
              GodotString::new()
            };
            edit.set_text(text);
            dialog.set_title(title);
            dialog.popup_centered(Vector2::zero());
            edit.grab_focus();
          }
        }
      }
    }
  }

  #[export]
  fn notes_changed(&self, owner: Node) {
    if let Some(mut edit) = owner.get_node_as::<TextEdit>(&self.notes_edit) {
      let text = unsafe { edit.get_text() };
      if let Some(avatar) = self.get_current_avatar(owner) {
        self.config.set_notes(avatar, Some(text));
      }
    }
  }

  fn get_avatars(&self) -> Vec<String> {
    self.data.borrow().get_avatars()
  }

  fn populate_avatars(&self, owner: Node) {
    if let Some(mut button) = owner.get_node_as::<OptionButton>(&self.avatars) {
      unsafe {
        self.enable_notes(owner, false);
        button.clear();

        let names = self.get_avatars();
        for (idx, name) in names.iter().enumerate() {
          button.add_item(GodotString::from(name), idx as i64 + 1);
        }

        if button.get_item_count() > 0 {
          if let Some(avatar) = self.config.get_avatar() {
            button.select_item(avatar);
          }

          let avatar = button.get_item_text(button.get_selected());
          if !avatar.is_empty() {
            self.enable_notes(owner, true);
            self.populate_dates(owner, Some(avatar.to_utf8().as_str()));
            return;
          }
        }
      }
    }
    self.populate_dates(owner, None);
  }

  fn get_current_avatar(&self, owner: Node) -> Option<GodotString> {
    unsafe {
      if let Some(button) = owner.get_node_as::<OptionButton>(&self.avatars) {
        let id = button.get_selected_id();
        if id != 0 {
          let avatar = button.get_item_text(button.get_item_index(id));
          if !avatar.is_empty() {
            return Some(avatar);
          }
        }
      }
    }
    None
  }

  fn get_stats_timestamps(&self, avatar: &str) -> Vec<i64> {
    self.data.borrow().get_stats_timestamps(avatar)
  }

  fn populate_dates(&self, owner: Node, avatar: Option<&str>) {
    if let Some(mut button) = owner.get_node_as::<OptionButton>(&self.dates) {
      unsafe {
        button.clear();
        if let Some(avatar) = avatar {
          let timestamps = self.get_stats_timestamps(avatar);
          if !timestamps.is_empty() {
            for ts in timestamps {
              let date = timestamp_to_view_date(ts);
              button.add_item(GodotString::from(date), ts);
            }

            let ts = button.get_item_id(0);
            if ts != 0 {
              self.populate_stats(owner, Some(avatar), Some(ts), StatOpts::None);
              return;
            }
          }
        }
      }
    }
    self.populate_stats(owner, avatar, None, StatOpts::None);
  }

  fn get_current_date(&self, owner: Node) -> Option<i64> {
    unsafe {
      if let Some(button) = owner.get_node_as::<OptionButton>(&self.dates) {
        let ts = button.get_selected_id();
        if ts != 0 {
          return Some(ts);
        }
      }
    }
    None
  }

  fn enable_notes(&self, owner: Node, enable: bool) {
    if let Some(mut button) = owner.get_node_as::<Button>(&self.notes) {
      unsafe {
        button.set_disabled(!enable);
        button.set_focus_mode(if enable {
          Control::FOCUS_ALL
        } else {
          Control::FOCUS_NONE
        });
      }
    }
  }

  fn get_stats(&self, avatar: &str, ts: i64) -> Option<StatsData> {
    self.data.borrow().get_stats(avatar, ts)
  }

  fn populate_stats(&self, owner: Node, avatar: Option<&str>, ts: Option<i64>, opts: StatOpts) {
    self.set_status_message(owner, None);
    unsafe {
      let mut tree = some!(owner.get_node_as::<Tree>(&self.tree));
      tree.clear();
      tree.set_focus_mode(Control::FOCUS_NONE as i64);

      let avatar = some!(avatar);
      if let Some(ts) = ts {
        if let Some(stats) = self.get_stats(avatar, ts) {
          if let Some(parent) = tree.create_item(None, -1) {
            let locale = get_locale();
            let mut bg_color = Cycle::new(vec![
              Color::rgb(0.18, 0.18, 0.18),
              Color::rgb(0.16, 0.16, 0.16),
            ]);

            match opts {
              StatOpts::Resists => {
                #[derive(Hash, Eq, PartialEq, Copy, Clone)]
                enum Resists {
                  Air,
                  Chaos,
                  Death,
                  Earth,
                  Fire,
                  Life,
                  Moon,
                  Sun,
                  Water,
                  Magic,
                }
                const RESIST_STATS: [(&str, (Resists, f64)); 19] = [
                  ("AirAttunement", (Resists::Air, 0.5)),
                  ("AirResistance", (Resists::Air, 1.0)),
                  ("ChaosAttunement", (Resists::Chaos, 0.5)),
                  ("ChaosResistance", (Resists::Chaos, 1.0)),
                  ("DeathAttunement", (Resists::Death, 0.5)),
                  ("DeathResistance", (Resists::Death, 1.0)),
                  ("EarthAttunement", (Resists::Earth, 0.5)),
                  ("EarthResistance", (Resists::Earth, 1.0)),
                  ("FireAttunement", (Resists::Fire, 0.5)),
                  ("FireResistance", (Resists::Fire, 1.0)),
                  ("LifeAttunement", (Resists::Life, 0.5)),
                  ("LifeResistance", (Resists::Life, 1.0)),
                  ("MoonAttunement", (Resists::Moon, 0.5)),
                  ("MoonResistance", (Resists::Moon, 1.0)),
                  ("SunAttunement", (Resists::Sun, 0.5)),
                  ("SunResistance", (Resists::Sun, 1.0)),
                  ("WaterAttunement", (Resists::Water, 0.5)),
                  ("WaterResistance", (Resists::Water, 1.0)),
                  ("MagicResistance", (Resists::Magic, 1.0)),
                ];
                const RESIST_NAMES: [&str; 9] = [
                  "Air", "Chaos", "Death", "Earth", "Fire", "Life", "Moon", "Sun", "Water",
                ];
                const RESIST_KEYS: [Resists; 9] = [
                  Resists::Air,
                  Resists::Chaos,
                  Resists::Death,
                  Resists::Earth,
                  Resists::Fire,
                  Resists::Life,
                  Resists::Moon,
                  Resists::Sun,
                  Resists::Water,
                ];
                let resist_stats: HashMap<&str, (Resists, f64)> =
                  RESIST_STATS.iter().cloned().collect();
                let mut resist_values: HashMap<Resists, f64> = HashMap::new();

                // Collect and sum the resistances.
                for (name, value) in stats.iter() {
                  if let Some((key, mul)) = resist_stats.get(name) {
                    // Stats possibly use ',' as the decimal separator depending on locale.
                    if let Ok(val) = value.replacen(',', ".", 1).parse::<f64>() {
                      if let Some(resist) = resist_values.get_mut(key) {
                        *resist += val * mul;
                      } else {
                        resist_values.insert(*key, val * mul);
                      }
                    }
                  }
                }

                // Add-in magic resistance.
                if let Some(magic) = resist_values.remove(&Resists::Magic) {
                  for (key, resist) in &mut resist_values {
                    if *key != Resists::Chaos {
                      *resist += magic;
                    }
                  }
                }

                // Format the output.
                for (pos, key) in RESIST_KEYS.iter().enumerate() {
                  if let Some(value) = resist_values.get(key) {
                    if let Some(mut item) = tree.create_item(parent.cast::<Object>(), -1) {
                      let name = RESIST_NAMES[pos];
                      let value = value.to_display_string(&locale);
                      let bg = *bg_color.get();

                      item.set_selectable(0, false);
                      item.set_selectable(1, false);
                      item.set_custom_bg_color(0, bg, false);
                      item.set_custom_bg_color(1, bg, false);
                      item.set_custom_color(0, Color::rgb(0.7, 0.6, 0.4));
                      item.set_text(0, GodotString::from(name));
                      item.set_text(1, GodotString::from(value));
                    }
                  }
                }

                let text = format!(
                  "Showing effective resists from {}",
                  timestamp_to_view_date(ts)
                );
                self.set_status_message(owner, Some(&text));
                tree.set_focus_mode(Control::FOCUS_ALL as i64);
                return;
              }
              _ => {
                for (name, value) in stats.iter() {
                  if let StatOpts::Filter(filter) = opts {
                    // Check if the name contains the filter string.
                    if !ascii_contains_ignore_case(name.as_bytes(), filter.as_bytes()) {
                      continue;
                    }
                  }

                  // Stats possibly use ',' as the decimal separator depending on locale.
                  if let Ok(value) = value.replacen(',', ".", 1).parse::<f64>() {
                    if let Some(mut item) = tree.create_item(parent.cast::<Object>(), -1) {
                      let value = &value.to_display_string(&locale);
                      let bg = *bg_color.get();

                      item.set_selectable(0, false);
                      item.set_selectable(1, false);
                      item.set_custom_bg_color(0, bg, false);
                      item.set_custom_bg_color(1, bg, false);
                      item.set_custom_color(0, Color::rgb(0.4, 0.6, 0.7));
                      item.set_text(0, GodotString::from(name));
                      item.set_text(1, GodotString::from(value));
                    }
                  }
                }

                let date = timestamp_to_view_date(ts);
                let text = match opts {
                  StatOpts::Filter(_) => format!("Showing filtered stats from {}", date),
                  _ => format!("Showing stats from {}", date),
                };
                self.set_status_message(owner, Some(&text));
                tree.set_focus_mode(Control::FOCUS_ALL as i64);
                return;
              }
            }
          }
        }
      }

      let text = format!("No stats found for {}", avatar);
      self.set_status_message(owner, Some(&text));
      return;
    }
  }

  fn set_status_message(&self, owner: Node, text: Option<&str>) {
    if let Some(mut label) = owner.get_node_as::<Label>(&self.status) {
      unsafe {
        match text {
          Some(text) => label.set_text(GodotString::from(text)),
          None => label.set_text(GodotString::new()),
        }
      }
    }
  }
}
