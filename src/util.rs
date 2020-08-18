use gdnative::api::*;
use gdnative::prelude::*;
use num_format::Locale;
use std::cmp::Ordering;

#[macro_export]
macro_rules! some {
  ($opt:expr) => {
    match $opt {
      Some(val) => val,
      None => return,
    }
  };
  ($opt:expr, $ret:expr) => {
    match $opt {
      Some(val) => val,
      None => return $ret,
    }
  };
}

#[macro_export]
macro_rules! ok {
  ($res:expr) => {
    match $res {
      Ok(val) => val,
      Err(err) => {
        godot_print!("{:?}", err);
        return;
      }
    }
  };
  ($res:expr, $ret:expr) => {
    match $res {
      Ok(val) => val,
      Err(err) => {
        godot_print!("{:?}", err);
        return $ret;
      }
    }
  };
}

pub struct Cycle<T> {
  index: usize,
  values: Vec<T>,
}

impl<T> Cycle<T> {
  pub fn new(values: Vec<T>) -> Self {
    assert!(!values.is_empty());
    Self {
      index: 0,
      values: values,
    }
  }

  pub fn get(&mut self) -> &T {
    let index = self.index;
    self.index = self.index + 1;
    if self.index >= self.values.len() {
      self.index = 0;
    }

    &self.values[index]
  }
}

pub trait OptionButtonText {
  fn find_item_index(self, text: &GodotString) -> Option<i64>;
  fn select_item(self, text: &GodotString) -> bool;
}

impl OptionButtonText for TRef<'_, OptionButton> {
  fn find_item_index(self, text: &GodotString) -> Option<i64> {
    let count = self.get_item_count();
    for index in 0..count {
      let item_text = self.get_item_text(index);
      if item_text == *text {
        return Some(index);
      }
    }
    None
  }

  fn select_item(self, text: &GodotString) -> bool {
    if let Some(index) = self.find_item_index(text) {
      self.select(index);
      return true;
    }
    false
  }
}

pub trait ToRef<'a, 'r, T>
where
  gdnative::object::AssumeSafeLifetime<'a, 'r>:
    gdnative::object::LifetimeConstraint<<T as GodotObject>::RefKind>,
  T: GodotObject + SubClass<Object>,
{
  fn to_ref(&'r self) -> TRef<'a, T, Shared>;
}

impl<'a, 'r, T> ToRef<'a, 'r, T> for Ref<T, Shared>
where
  gdnative::object::AssumeSafeLifetime<'a, 'r>:
    gdnative::object::LifetimeConstraint<<T as GodotObject>::RefKind>,
  T: GodotObject + SubClass<Object>,
{
  fn to_ref(&'r self) -> TRef<'a, T, Shared> {
    unsafe { self.assume_safe() }
  }
}

pub trait GetNodeAs {
  fn get_node_as<T>(self, path: &GodotString) -> Option<TRef<'_, T, Shared>>
  where
    T: GodotObject + SubClass<Node>;
}

impl GetNodeAs for TRef<'_, Node> {
  fn get_node_as<T>(self, path: &GodotString) -> Option<TRef<'_, T, Shared>>
  where
    T: GodotObject + SubClass<Node>,
  {
    if let Some(node) = self.get_node(NodePath::new(path)) {
      let node = node.to_ref().cast();
      if node.is_none() {
        godot_print!(
          "Unable to cast node {} as {:?}",
          path,
          std::any::type_name::<T>()
        );
      }
      return node;
    } else {
      godot_print!("Unable to get node {}", path);
    }
    None
  }
}

pub trait ConnectTo {
  fn connect_to(self, path: &GodotString, signal: &str, slot: &str) -> bool;
}

impl ConnectTo for TRef<'_, Node> {
  fn connect_to(self, path: &GodotString, signal: &str, slot: &str) -> bool {
    if let Some(node) = self.get_node(NodePath::new(path)) {
      let mut node = node.to_ref();

      // Get the popup if this is a menu button.
      if let Some(button) = node.cast::<MenuButton>() {
        if let Some(popup) = button.get_popup() {
          node = popup.to_ref().upcast::<Node>();
        } else {
          godot_print!("Unable to get popup for {}", path);
          return false;
        }
      }

      if let Err(err) = node.connect(signal, self, slot, VariantArray::new_shared(), 0) {
        godot_print!("Unable to connect {}: {:?}", slot, err);
      } else {
        return true;
      }
    } else {
      godot_print!("Unable to get node {}", path);
    }
    false
  }
}

pub trait SetShortcut {
  fn set_shortcut(self, id: i64, key: i64, ctrl: bool);
}

impl SetShortcut for TRef<'_, PopupMenu> {
  fn set_shortcut(self, id: i64, key: i64, ctrl: bool) {
    let input = InputEventKey::new();
    input.set_control(ctrl);
    input.set_scancode(key);
    self.set_item_accelerator(self.get_item_index(id), input.get_scancode_with_modifiers());
  }
}

pub struct Config {
  log_path: Option<GodotString>,
  cfg_path: GodotString,
  section: GodotString,
  folder_key: GodotString,
  avatar_key: GodotString,
}

impl Config {
  pub fn new() -> Config {
    let mut log_path = None;
    if let Some(dir) = dirs::config_dir() {
      let path = dir.join("Portalarium/Shroud of the Avatar/ChatLogs");
      if let Some(path) = path.to_str() {
        let path = if cfg!(target_os = "windows") {
          // Change any backslashes to forward slashes.
          GodotString::from(path.replace('\\', "/"))
        } else {
          GodotString::from(path)
        };
        log_path = Some(path);
      }
    }

    Config {
      log_path: log_path,
      cfg_path: GodotString::from("user://settings.cfg"),
      section: GodotString::from("main"),
      folder_key: GodotString::from("log_folder"),
      avatar_key: GodotString::from("avatar"),
    }
  }

  fn notes_key(avatar: &GodotString) -> GodotString {
    GodotString::from(format!(
      "{}_notes",
      avatar.to_utf8().as_str().replace(' ', "_")
    ))
  }

  pub fn get_log_folder(&self) -> Option<GodotString> {
    if let Some(folder) = self.get_value(&self.folder_key) {
      return Some(folder);
    } else if let Some(folder) = &self.log_path {
      return Some(folder.clone());
    }
    None
  }

  pub fn set_log_folder(&self, folder: Option<&GodotString>) {
    self.set_value(&self.folder_key, folder);
  }

  pub fn get_avatar(&self) -> Option<GodotString> {
    self.get_value(&self.avatar_key)
  }

  pub fn set_avatar(&self, avatar: Option<&GodotString>) {
    self.set_value(&self.avatar_key, avatar);
  }

  pub fn get_notes(&self, avatar: &GodotString) -> Option<GodotString> {
    if !avatar.is_empty() {
      return self.get_value(&Config::notes_key(avatar));
    }
    None
  }

  pub fn set_notes(&self, avatar: &GodotString, notes: Option<&GodotString>) {
    if !avatar.is_empty() {
      self.set_value(&Config::notes_key(avatar), notes);
    }
  }

  fn get_value(&self, key: &GodotString) -> Option<GodotString> {
    let config = ConfigFile::new();
    if !self.cfg_path.is_empty() && config.load(self.cfg_path.clone()).is_ok() {
      if config.has_section_key(self.section.clone(), key.clone()) {
        let value = config.get_value(self.section.clone(), key.clone(), Variant::new());
        if !value.is_nil() {
          return Some(value.to_godot_string());
        }
      }
    }
    None
  }

  fn set_value(&self, key: &GodotString, value: Option<&GodotString>) {
    let config = ConfigFile::new();
    let _ = config.load(self.cfg_path.clone());
    if let Some(value) = value {
      config.set_value(
        self.section.clone(),
        key.clone(),
        Variant::from_godot_string(&value),
      );
    } else if config.has_section_key(self.section.clone(), key.clone()) {
      config.erase_section_key(self.section.clone(), key.clone());
    }
    let _ = config.save(self.cfg_path.clone());
  }
}

pub fn ascii_starts_with_ignore_case(container: &[u8], pattern: &[u8]) -> bool {
  if pattern.is_empty() || container.len() < pattern.len() {
    return false;
  }

  for index in 0..pattern.len() {
    if container[index].to_ascii_lowercase() != pattern[index].to_ascii_lowercase() {
      return false;
    }
  }

  return true;
}

pub fn ascii_contains_ignore_case(container: &[u8], pattern: &[u8]) -> bool {
  if !pattern.is_empty() {
    let mut container = container;
    while container.len() >= pattern.len() {
      if ascii_starts_with_ignore_case(container, pattern) {
        return true;
      }
      container = &container[1..];
    }
  }

  false
}

pub fn _ascii_equals_ignore_case(left: &[u8], right: &[u8]) -> bool {
  left.len() == right.len() && ascii_starts_with_ignore_case(left, right)
}

pub fn ascii_compare_ignore_case(left: &[u8], right: &[u8]) -> Ordering {
  let mut il = left.iter();
  let mut ir = right.iter();
  loop {
    if let Some(cl) = il.next() {
      if let Some(cr) = ir.next() {
        match cl.to_ascii_lowercase().cmp(&cr.to_ascii_lowercase()) {
          Ordering::Less => return Ordering::Less,
          Ordering::Equal => continue,
          Ordering::Greater => return Ordering::Greater,
        }
      }
    }
    return left.len().cmp(&right.len());
  }
}

pub fn get_locale() -> Locale {
  let names = Locale::available_names();
  let name = OS::godot_singleton()
    .get_locale()
    .to_utf8()
    .as_str()
    .replace('_', "-");

  // Search for an exact match.
  if let Ok(pos) =
    names.binary_search_by(|n| ascii_compare_ignore_case(n.as_bytes(), name.as_bytes()))
  {
    if let Ok(locale) = Locale::from_name(names[pos]) {
      return locale;
    }
  } else {
    // Exact match not found, try the base language.
    if let Some(name) = name.split('-').next() {
      if let Ok(locale) = Locale::from_name(name) {
        return locale;
      }
    }
  }

  Locale::en
}

pub trait ToDisplayString {
  fn to_display_string(self, locale: &Locale) -> String;
}

impl ToDisplayString for f64 {
  fn to_display_string(self, locale: &Locale) -> String {
    format!("{:.6}", self)
      .trim_end_matches('0')
      .trim_end_matches('.')
      .replacen('.', locale.decimal(), 1)
  }
}
