use crate::util::*;
use gdnative::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Stats {
  data: LogData,
}

#[methods]
impl Stats {
  fn _init(_owner: Node) -> Self {
    let folder = match dirs::config_dir() {
      Some(folder) => folder.join("Portalarium/Shroud of the Avatar/ChatLogs"),
      None => std::path::PathBuf::new(),
    };
    Stats {
      data: LogData::new(folder),
    }
  }

  #[export]
  fn _ready(&mut self, owner: Node) {
    unsafe {
      let object = &owner.to_object();
      let signal = GodotString::from_str("item_selected");
      if let Some(mut avatars) = owner.get_avatars() {
        let _ = avatars.connect(
          signal.new_ref(),
          Some(*object),
          GodotString::from_str("avatar_changed"),
          VariantArray::new(),
          0,
        );
      }

      if let Some(mut dates) = owner.get_dates() {
        let _ = dates.connect(
          signal.new_ref(),
          Some(*object),
          GodotString::from_str("date_changed"),
          VariantArray::new(),
          0,
        );
      }
      if let Some(mut tree) = owner.get_stats() {
        tree.set_column_expand(0, true);
        tree.set_column_min_width(0, 3);
      }
    }
    owner.populate_avatars(&self.data);
  }

  #[export]
  fn avatar_changed(&mut self, owner: Node, item: i64) {
    if let Some(avatars) = owner.get_avatars() {
      unsafe {
        let avatar = avatars.get_item_text(item);
        if !avatar.is_empty() {
          owner.populate_dates(&self.data, Some(avatar.to_utf8().as_str()));
          return;
        }
      }
    }
    owner.populate_dates(&self.data, None);
  }

  #[export]
  fn date_changed(&mut self, owner: Node, item: i64) {
    if let Some(avatar) = owner.get_current_avatar() {
      if let Some(dates) = owner.get_dates() {
        unsafe {
          let ts = dates.get_item_id(item);
          if ts != 0 {
            owner.populate_stats(&self.data, Some(avatar.to_utf8().as_str()), Some(ts));
            return;
          }
        }
      }
    }
    owner.populate_stats(&self.data, None, None);
  }
}

trait StatsNode {
  fn get_avatars(&self) -> Option<OptionButton>;
  fn populate_avatars(&self, data: &LogData);
  fn get_current_avatar(&self) -> Option<GodotString>;
  fn get_dates(&self) -> Option<OptionButton>;
  fn populate_dates(&self, data: &LogData, avatar: Option<&str>);
  fn get_current_date(&self) -> Option<i64>;
  fn get_stats(&self) -> Option<Tree>;
  fn populate_stats(&self, data: &LogData, avatar: Option<&str>, ts: Option<i64>);
  fn get_status(&self) -> Option<Label>;
  fn set_status_message(&self, text: Option<&str>);
}

impl StatsNode for Node {
  fn get_avatars(&self) -> Option<OptionButton> {
    unsafe {
      if let Some(avatars) = self.get_node(NodePath::from_str("Tools/Avatars")) {
        return avatars.cast::<OptionButton>();
      }
    }
    None
  }

  fn populate_avatars(&self, data: &LogData) {
    if let Some(mut avatars) = self.get_avatars() {
      unsafe {
        avatars.clear();

        let names = data.get_avatars();
        for (idx, name) in names.iter().enumerate() {
          avatars.add_item(GodotString::from_str(&name), idx as i64 + 1);
        }

        let avatar = avatars.get_item_text(0);
        if !avatar.is_empty() {
          self.populate_dates(data, Some(avatar.to_utf8().as_str()));
          return;
        }
      }
    }
    self.populate_dates(data, None);
  }

  fn get_current_avatar(&self) -> Option<GodotString> {
    unsafe {
      if let Some(avatars) = self.get_avatars() {
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

  fn get_dates(&self) -> Option<OptionButton> {
    unsafe {
      if let Some(dates) = self.get_node(NodePath::from_str("Tools/Dates")) {
        return dates.cast::<OptionButton>();
      }
    }
    None
  }

  fn populate_dates(&self, data: &LogData, avatar: Option<&str>) {
    if let Some(mut dates) = self.get_dates() {
      unsafe {
        dates.clear();
        if let Some(avatar) = avatar {
          let timestamps = data.get_stats_timestamps(avatar);
          if !timestamps.is_empty() {
            for ts in timestamps {
              let date = timestamp_to_view_date(ts);
              dates.add_item(GodotString::from_str(&date), ts);
            }

            let ts = dates.get_item_id(0);
            if ts != 0 {
              self.populate_stats(data, Some(avatar), Some(ts));
              return;
            }
          }
        }
      }
    }
    self.populate_stats(data, avatar, None);
  }

  fn get_current_date(&self) -> Option<i64> {
    unsafe {
      if let Some(dates) = self.get_dates() {
        let ts = dates.get_selected_id();
        if ts != 0 {
          return Some(ts);
        }
      }
    }
    None
  }

  fn get_stats(&self) -> Option<Tree> {
    unsafe {
      if let Some(stats) = self.get_node(NodePath::from_str("Tree")) {
        return stats.cast::<Tree>();
      }
    }
    None
  }

  fn populate_stats(&self, data: &LogData, avatar: Option<&str>, ts: Option<i64>) {
    self.set_status_message(None);
    unsafe {
      let mut tree = some!(self.get_stats());
      tree.clear();

      let avatar = some!(avatar);
      let ts = some!(ts);

      if let Some(stats) = data.get_stats(avatar, ts) {
        if let Some(parent) = tree.create_item(None, -1) {
          let locale = get_locale();
          for (name, value) in stats.iter() {
            if let Ok(num) = value.replacen(',', ".", 1).parse::<f64>() {
              if let Some(mut item) = tree.create_item(parent.cast::<Object>(), -1) {
                let value = &num.to_display_string(&locale);
                let tip = GodotString::from_str(&format!("{} = {}", name, value));
                item.set_tooltip(0, tip.new_ref());
                item.set_tooltip(1, tip);
                item.set_custom_color(0, Color::rgb(0.4, 0.6, 0.7));
                item.set_text(0, GodotString::from_str(name));
                item.set_text(1, GodotString::from_str(&value));
              }
            }
          }
        }
        let date = timestamp_to_view_date(ts);
        let text = t!("Showing stats from {}").replacen("{}", &date, 1);
        self.set_status_message(Some(&text));
        return;
      }
      let text = t!("No stats found for {}").replacen("{}", avatar, 1);
      self.set_status_message(Some(&text));
      return;
    }
  }

  fn get_status(&self) -> Option<Label> {
    unsafe {
      if let Some(status) = self.get_node(NodePath::from_str("Status")) {
        return status.cast::<Label>();
      }
    }
    None
  }

  fn set_status_message(&self, text: Option<&str>) {
    unsafe {
      if let Some(mut status) = self.get_status() {
        match text {
          Some(text) => status.set_text(GodotString::from_str(text)),
          None => status.set_text(GodotString::new()),
        }
      }
    }
  }
}
