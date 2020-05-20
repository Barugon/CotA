use crate::util::*;
use gdnative::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Stats {
  config: Config,
  data: LogData,
  avatars: NodePath,
  dates: NodePath,
  list: NodePath,
  status: NodePath,
  file_dialog: NodePath,
}

#[methods]
impl Stats {
  fn _init(_owner: Node) -> Self {
    let config = Config::new();
    let folder = config.get_log_folder();
    Stats {
      config: config,
      data: LogData::new(folder),
      avatars: NodePath::from_str("Tools/Avatars"),
      dates: NodePath::from_str("Tools/Dates"),
      list: NodePath::from_str("List"),
      status: NodePath::from_str("Status"),
      file_dialog: NodePath::from_str("/root/Main/FileDialog"),
    }
  }

  #[export]
  fn _ready(&mut self, owner: Node) {
    unsafe {
      let object = &owner.to_object();
      let signal = GodotString::from_str("item_selected");
      if let Some(mut avatars) = self.get_avatars(owner) {
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

      if let Some(mut dates) = self.get_dates(owner) {
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

      if let Some(mut file_dialog) = self.get_file_dialog(owner) {
        if let Err(err) = file_dialog.connect(
          GodotString::from_str("dir_selected"),
          Some(*object),
          GodotString::from_str("set_log_folder"),
          VariantArray::new(),
          0,
        ) {
          godot_print!("Unable to connect set_log_folder: {:?}", err);
        }
      }

      if let Some(mut list) = self.get_list(owner) {
        list.set_column_expand(0, true);
        list.set_column_min_width(0, 3);
      }
    }
    self.populate_avatars(owner);
  }

  #[export]
  fn avatar_changed(&mut self, owner: Node, item: i64) {
    if let Some(avatars) = self.get_avatars(owner) {
      unsafe {
        let avatar = avatars.get_item_text(item);
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
      if let Some(dates) = self.get_dates(owner) {
        unsafe {
          let ts = dates.get_item_id(item);
          if ts != 0 {
            self.populate_list(owner, Some(avatar.to_utf8().as_str()), Some(ts));
            return;
          }
        }
      }
    }
    self.populate_list(owner, None, None);
  }

  #[export]
  fn set_log_folder(&mut self, owner: Node, folder: GodotString) {
    self.data = LogData::new(folder.new_ref());
    self.config.set_log_folder(folder);
    self.populate_avatars(owner);
  }

  fn get_avatars(&self, owner: Node) -> Option<OptionButton> {
    unsafe {
      if let Some(avatars) = owner.get_node(self.avatars.new_ref()) {
        let avatars = avatars.cast::<OptionButton>();
        if avatars.is_none() {
          godot_print!("Unable to cast node Avatars as OptionButton");
        }
        return avatars;
      } else {
        godot_print!("Unable to get node Avatars");
      }
    }
    None
  }

  fn populate_avatars(&self, owner: Node) {
    if let Some(mut avatars) = self.get_avatars(owner) {
      unsafe {
        avatars.clear();

        let names = self.data.get_avatars();
        for (idx, name) in names.iter().enumerate() {
          avatars.add_item(GodotString::from_str(&name), idx as i64 + 1);
        }

        if avatars.get_item_count() > 0 {
          let avatar = avatars.get_item_text(0);
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
      if let Some(avatars) = self.get_avatars(owner) {
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

  fn get_dates(&self, owner: Node) -> Option<OptionButton> {
    unsafe {
      if let Some(dates) = owner.get_node(self.dates.new_ref()) {
        let dates = dates.cast::<OptionButton>();
        if dates.is_none() {
          godot_print!("Unable to cast node Dates as OptionButton");
        }
        return dates;
      } else {
        godot_print!("Unable to get node Dates");
      }
    }
    None
  }

  fn populate_dates(&self, owner: Node, avatar: Option<&str>) {
    if let Some(mut dates) = self.get_dates(owner) {
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
              self.populate_list(owner, Some(avatar), Some(ts));
              return;
            }
          }
        }
      }
    }
    self.populate_list(owner, avatar, None);
  }

  fn _get_current_date(&self, owner: Node) -> Option<i64> {
    unsafe {
      if let Some(dates) = self.get_dates(owner) {
        let ts = dates.get_selected_id();
        if ts != 0 {
          return Some(ts);
        }
      }
    }
    None
  }

  fn get_list(&self, owner: Node) -> Option<Tree> {
    unsafe {
      if let Some(list) = owner.get_node(self.list.new_ref()) {
        let list = list.cast::<Tree>();
        if list.is_none() {
          godot_print!("Unable to cast node List as Tree");
        }
        return list;
      } else {
        godot_print!("Unable to get node List");
      }
    }
    None
  }

  fn populate_list(&self, owner: Node, avatar: Option<&str>, ts: Option<i64>) {
    self.set_status_message(owner, None);
    unsafe {
      let mut list = some!(self.get_list(owner));
      list.clear();

      let avatar = some!(avatar);
      let ts = some!(ts);

      if let Some(stats) = self.data.get_stats(avatar, ts) {
        if let Some(parent) = list.create_item(None, -1) {
          let locale = get_locale();
          let mut alt_bg = false;
          for (name, value) in stats.iter() {
            if let Ok(num) = value.replacen(',', ".", 1).parse::<f64>() {
              if let Some(mut item) = list.create_item(parent.cast::<Object>(), -1) {
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
        }
        let text = format!("Showing stats from {}", timestamp_to_view_date(ts));
        self.set_status_message(owner, Some(&text));
        return;
      }
      let text = format!("No stats found for {}", avatar);
      self.set_status_message(owner, Some(&text));
      return;
    }
  }

  fn get_status(&self, owner: Node) -> Option<Label> {
    unsafe {
      if let Some(status) = owner.get_node(self.status.new_ref()) {
        let status = status.cast::<Label>();
        if status.is_none() {
          godot_print!("Unable to cast node Status as Label");
        }
        return status;
      } else {
        godot_print!("Unable to get node Status");
      }
    }
    None
  }

  fn set_status_message(&self, owner: Node, text: Option<&str>) {
    unsafe {
      if let Some(mut status) = self.get_status(owner) {
        match text {
          Some(text) => status.set_text(GodotString::from_str(text)),
          None => status.set_text(GodotString::new()),
        }
      }
    }
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
