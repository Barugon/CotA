use crate::menu::*;
use crate::util::*;
use gdnative::*;
use std::collections::HashMap;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Stats {
  config: Config,
  data: LogData,
  avatars: NodePath,
  dates: NodePath,
  tree: NodePath,
  status: NodePath,
  folder_dialog: NodePath,
  filter_dialog: NodePath,
  filter_edit: NodePath,
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
      data: LogData::new(folder),
      avatars: NodePath::from_str("Tools/Avatars"),
      dates: NodePath::from_str("Tools/Dates"),
      tree: NodePath::from_str("Tree"),
      status: NodePath::from_str("Status"),
      folder_dialog: NodePath::from_str("/root/Main/FolderDialog"),
      filter_dialog: NodePath::from_str("/root/Main/FilterDialog"),
      filter_edit: NodePath::from_str("/root/Main/FilterDialog/VBoxContainer/FilterText"),
    }
  }

  #[export]
  fn _ready(&mut self, owner: Node) {
    unsafe {
      let object = &owner.to_object();
      let signal = GodotString::from_str("item_selected");

      // Connect the avatars button.
      if let Some(mut avatars) = self.get_avatars_button(owner) {
        if let Err(err) = avatars.connect(
          signal.new_ref(),
          Some(*object),
          GodotString::from_str("avatar_changed"),
          VariantArray::new(),
          0,
        ) {
          godot_print!("Unable to connect avatar_changed: {:?}", err);
        }
      }

      // Connect the dates button.
      if let Some(mut dates) = self.get_dates_button(owner) {
        if let Err(err) = dates.connect(
          signal.new_ref(),
          Some(*object),
          GodotString::from_str("date_changed"),
          VariantArray::new(),
          0,
        ) {
          godot_print!("Unable to connect date_changed: {:?}", err);
        }
      }

      // Connect the log folder dialog.
      if let Some(mut dialog) = self.get_folder_dialog(owner) {
        if let Err(err) = dialog.connect(
          GodotString::from_str("dir_selected"),
          Some(*object),
          GodotString::from_str("log_folder_changed"),
          VariantArray::new(),
          0,
        ) {
          godot_print!("Unable to connect log_folder_changed: {:?}", err);
        }
      }

      // Connect the filter dialog.
      if let Some(mut dialog) = self.get_filter_dialog(owner) {
        dialog.register_text_enter(owner.get_node(self.filter_edit.new_ref()));
        if let Err(err) = dialog.connect(
          GodotString::from_str("confirmed"),
          Some(*object),
          GodotString::from_str("filter_changed"),
          VariantArray::new(),
          0,
        ) {
          godot_print!("Unable to connect filter_changed: {:?}", err);
        }
      }

      // Set some stats tree properties.
      if let Some(mut tree) = self.get_stats_tree(owner) {
        tree.set_column_expand(0, true);
        tree.set_column_min_width(0, 3);
      }
    }
    self.populate_avatars(owner);
  }

  #[export]
  fn view_menu_select(&mut self, owner: Node, id: i64) {
    match id {
      REFRESH_ID => self.populate_avatars(owner),
      RESISTS_ID => {
        if let Some(avatar) = self.get_current_avatar(owner) {
          if let Some(ts) = self.get_current_date(owner) {
            self.populate_stats_tree(
              owner,
              Some(avatar.to_utf8().as_str()),
              Some(ts),
              StatOpts::Resists,
            );
          }
        }
      }
      FILTER_ID => {
        if let Some(mut dialog) = self.get_filter_dialog(owner) {
          if let Some(mut text) = self.get_filter_edit(owner) {
            unsafe {
              text.set_text(GodotString::new());
              dialog.popup_centered(Vector2::zero());
              text.grab_focus();
            }
          }
        }
      }
      RESET_ID => {
        if !self.close_dialogs(owner) {
          if let Some(avatar) = self.get_current_avatar(owner) {
            if let Some(ts) = self.get_current_date(owner) {
              self.populate_stats_tree(
                owner,
                Some(avatar.to_utf8().as_str()),
                Some(ts),
                StatOpts::None,
              );
            }
          }
        }
      }
      _ => {}
    }
  }

  #[export]
  fn avatar_changed(&mut self, owner: Node, item: i64) {
    if let Some(avatars) = self.get_avatars_button(owner) {
      unsafe {
        let avatar = avatars.get_item_text(item);
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
  fn date_changed(&mut self, owner: Node, item: i64) {
    if let Some(avatar) = self.get_current_avatar(owner) {
      if let Some(dates) = self.get_dates_button(owner) {
        unsafe {
          let ts = dates.get_item_id(item);
          if ts != 0 {
            self.populate_stats_tree(
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
    self.populate_stats_tree(owner, None, None, StatOpts::None);
  }

  #[export]
  fn log_folder_changed(&mut self, owner: Node, folder: GodotString) {
    self.data = LogData::new(folder.new_ref());
    self.config.set_log_folder(Some(folder));
    self.populate_avatars(owner);
  }

  #[export]
  fn filter_changed(&self, owner: Node) {
    if let Some(text) = self.get_filter_edit(owner) {
      let text = unsafe { text.get_text() };
      if !text.is_empty() {
        if let Some(avatar) = self.get_current_avatar(owner) {
          if let Some(ts) = self.get_current_date(owner) {
            self.populate_stats_tree(
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

  fn get_avatars_button(&self, owner: Node) -> Option<OptionButton> {
    unsafe {
      if let Some(node) = owner.get_node(self.avatars.new_ref()) {
        let button = node.cast::<OptionButton>();
        if button.is_none() {
          godot_print!("Unable to cast node Avatars as OptionButton");
        }
        return button;
      } else {
        godot_print!("Unable to get node Avatars");
      }
    }
    None
  }

  fn populate_avatars(&self, owner: Node) {
    if let Some(mut avatars) = self.get_avatars_button(owner) {
      unsafe {
        avatars.clear();

        let names = self.data.get_avatars();
        for (idx, name) in names.iter().enumerate() {
          avatars.add_item(GodotString::from_str(&name), idx as i64 + 1);
        }

        if avatars.get_item_count() > 0 {
          if let Some(avatar) = self.config.get_avatar() {
            avatars.select_item(avatar);
          }

          let avatar = avatars.get_item_text(avatars.get_selected());
          if !avatar.is_empty() {
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
      if let Some(avatars) = self.get_avatars_button(owner) {
        let id = avatars.get_selected_id();
        if id != 0 {
          let avatar = avatars.get_item_text(avatars.get_item_index(id));
          if !avatar.is_empty() {
            return Some(avatar);
          }
        }
      }
    }
    None
  }

  fn get_dates_button(&self, owner: Node) -> Option<OptionButton> {
    unsafe {
      if let Some(node) = owner.get_node(self.dates.new_ref()) {
        let button = node.cast::<OptionButton>();
        if button.is_none() {
          godot_print!("Unable to cast node Dates as OptionButton");
        }
        return button;
      } else {
        godot_print!("Unable to get node Dates");
      }
    }
    None
  }

  fn populate_dates(&self, owner: Node, avatar: Option<&str>) {
    if let Some(mut dates) = self.get_dates_button(owner) {
      unsafe {
        dates.clear();
        if let Some(avatar) = avatar {
          let timestamps = self.data.get_stats_timestamps(avatar);
          if !timestamps.is_empty() {
            for ts in timestamps {
              let date = timestamp_to_view_date(ts);
              dates.add_item(GodotString::from_str(&date), ts);
            }

            let ts = dates.get_item_id(0);
            if ts != 0 {
              self.populate_stats_tree(owner, Some(avatar), Some(ts), StatOpts::None);
              return;
            }
          }
        }
      }
    }
    self.populate_stats_tree(owner, avatar, None, StatOpts::None);
  }

  fn get_current_date(&self, owner: Node) -> Option<i64> {
    unsafe {
      if let Some(dates) = self.get_dates_button(owner) {
        let ts = dates.get_selected_id();
        if ts != 0 {
          return Some(ts);
        }
      }
    }
    None
  }

  fn get_stats_tree(&self, owner: Node) -> Option<Tree> {
    unsafe {
      if let Some(node) = owner.get_node(self.tree.new_ref()) {
        let tree = node.cast::<Tree>();
        if tree.is_none() {
          godot_print!("Unable to cast node Tree as Tree");
        }
        return tree;
      } else {
        godot_print!("Unable to get node Tree");
      }
    }
    None
  }

  fn populate_stats_tree(
    &self,
    owner: Node,
    avatar: Option<&str>,
    ts: Option<i64>,
    opts: StatOpts,
  ) {
    self.set_status_message(owner, None);
    unsafe {
      let mut tree = some!(self.get_stats_tree(owner));
      tree.clear();

      let avatar = some!(avatar);
      if let Some(ts) = ts {
        if let Some(stats) = self.data.get_stats(avatar, ts) {
          if let Some(parent) = tree.create_item(None, -1) {
            let locale = get_locale();
            let mut alt_bg = false;

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
                    if let Ok(mut val) = value.replacen(',', ".", 1).parse::<f64>() {
                      val *= mul;

                      if let Some(resist) = resist_values.get(key) {
                        val += resist;
                      }

                      resist_values.insert(*key, val);
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
                      let tip = GodotString::from_str(&format!("{} = {}", name, value));
                      let bg = if alt_bg {
                        alt_bg = false;
                        Color::rgb(0.16, 0.16, 0.16)
                      } else {
                        alt_bg = true;
                        Color::rgb(0.18, 0.18, 0.18)
                      };
                      item.set_selectable(0, false);
                      item.set_selectable(1, false);
                      item.set_custom_bg_color(0, bg, false);
                      item.set_custom_bg_color(1, bg, false);
                      item.set_custom_color(0, Color::rgb(0.7, 0.6, 0.4));
                      item.set_tooltip(0, tip.new_ref());
                      item.set_tooltip(1, tip);
                      item.set_text(0, GodotString::from_str(name));
                      item.set_text(1, GodotString::from_str(&value));
                    }
                  }
                }

                let text = format!(
                  "Showing effective resists from {}",
                  timestamp_to_view_date(ts)
                );
                self.set_status_message(owner, Some(&text));
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

                  if let Ok(num) = value.replacen(',', ".", 1).parse::<f64>() {
                    if let Some(mut item) = tree.create_item(parent.cast::<Object>(), -1) {
                      let value = &num.to_display_string(&locale);
                      let tip = GodotString::from_str(&format!("{} = {}", name, value));
                      let bg = if alt_bg {
                        alt_bg = false;
                        Color::rgb(0.16, 0.16, 0.16)
                      } else {
                        alt_bg = true;
                        Color::rgb(0.18, 0.18, 0.18)
                      };
                      item.set_selectable(0, false);
                      item.set_selectable(1, false);
                      item.set_custom_bg_color(0, bg, false);
                      item.set_custom_bg_color(1, bg, false);
                      item.set_custom_color(0, Color::rgb(0.4, 0.6, 0.7));
                      item.set_tooltip(0, tip.new_ref());
                      item.set_tooltip(1, tip);
                      item.set_text(0, GodotString::from_str(name));
                      item.set_text(1, GodotString::from_str(&value));
                    }
                  }
                }

                let text = format!("Showing stats from {}", timestamp_to_view_date(ts));
                self.set_status_message(owner, Some(&text));
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

  fn get_status_label(&self, owner: Node) -> Option<Label> {
    unsafe {
      if let Some(node) = owner.get_node(self.status.new_ref()) {
        let label = node.cast::<Label>();
        if label.is_none() {
          godot_print!("Unable to cast node Status as Label");
        }
        return label;
      } else {
        godot_print!("Unable to get node Status");
      }
    }
    None
  }

  fn set_status_message(&self, owner: Node, text: Option<&str>) {
    unsafe {
      if let Some(mut status) = self.get_status_label(owner) {
        match text {
          Some(text) => status.set_text(GodotString::from_str(text)),
          None => status.set_text(GodotString::new()),
        }
      }
    }
  }

  fn get_folder_dialog(&self, owner: Node) -> Option<FileDialog> {
    unsafe {
      if let Some(dialog) = owner.get_node(self.folder_dialog.new_ref()) {
        let dialog = dialog.cast::<FileDialog>();
        if dialog.is_none() {
          godot_print!("Unable to cast node FolderDialog as FileDialog");
        }
        return dialog;
      } else {
        godot_print!("Unable to get node FolderDialog");
      }
    }
    None
  }

  fn get_filter_dialog(&self, owner: Node) -> Option<ConfirmationDialog> {
    unsafe {
      if let Some(dialog) = owner.get_node(self.filter_dialog.new_ref()) {
        let dialog = dialog.cast::<ConfirmationDialog>();
        if dialog.is_none() {
          godot_print!("Unable to cast node FilterDialog as WindowDialog");
        }
        return dialog;
      } else {
        godot_print!("Unable to get node FilterDialog");
      }
    }
    None
  }

  fn get_filter_edit(&self, owner: Node) -> Option<LineEdit> {
    unsafe {
      if let Some(node) = owner.get_node(self.filter_edit.new_ref()) {
        let edit = node.cast::<LineEdit>();
        if edit.is_none() {
          godot_print!("Unable to cast node FilterText as LineEdit");
        }
        return edit;
      } else {
        godot_print!("Unable to get node FilterText");
      }
    }
    None
  }

  fn close_dialogs(&self, owner: Node) -> bool {
    if let Some(mut dialog) = self.get_folder_dialog(owner) {
      unsafe {
        if dialog.is_visible() {
          dialog.hide();
          return true;
        }
      }
    }

    if let Some(mut dialog) = self.get_filter_dialog(owner) {
      unsafe {
        if dialog.is_visible() {
          dialog.hide();
          return true;
        }
      }
    }

    false
  }
}
