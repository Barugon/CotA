use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use gdnative::*;
use num_cpus;
use num_format::Locale;
use regex::Regex;
use std::{
  cell::RefCell,
  cmp::Ordering,
  collections::HashSet,
  fs,
  path::{Path, PathBuf},
  str::SplitWhitespace,
};
use thread_pool::*;

#[macro_export]
macro_rules! some {
  ($opt:expr) => {
    if let Some(val) = $opt {
      val
    } else {
      return;
    }
  };
  ($opt:expr, $ret:expr) => {
    if let Some(val) = $opt {
      val
    } else {
      return $ret;
    }
  };
}

#[macro_export]
macro_rules! ok {
  ($res:expr) => {{
    let val = $res;
    if let Ok(val) = val {
      val
    } else {
      if let Err(err) = val {
        println!("{:?}", err);
      }
      return;
    }
  }};
  ($res:expr, $ret:expr) => {{
    let val = $res;
    if let Ok(val) = val {
      val
    } else {
      if let Err(err) = val {
        println!("{:?}", err);
      }
      return $ret;
    }
  }};
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
  fn find_item_index(&self, text: GodotString) -> Option<i64>;
  fn select_item(&mut self, text: GodotString) -> bool;
}

impl OptionButtonText for OptionButton {
  fn find_item_index(&self, text: GodotString) -> Option<i64> {
    let count = unsafe { self.get_item_count() };
    for index in 0..count {
      let item_text = unsafe { self.get_item_text(index) };
      if item_text == text {
        return Some(index);
      }
    }
    None
  }

  fn select_item(&mut self, text: GodotString) -> bool {
    if let Some(index) = self.find_item_index(text) {
      unsafe {
        self.select(index);
      }
      return true;
    }
    false
  }
}

pub trait GetNodeAs {
  fn get_node_as<T: GodotObject>(self, path: &NodePath) -> Option<T>;
}

impl GetNodeAs for Node {
  fn get_node_as<T: GodotObject>(self, path: &NodePath) -> Option<T> {
    unsafe {
      if let Some(node) = self.get_node(path.new_ref()) {
        let node = node.cast::<T>();
        if node.is_none() {
          let name = path.to_godot_string();
          godot_print!(
            "Unable to cast node {} as {:?}",
            name.to_utf8().as_str(),
            std::any::type_name::<T>()
          );
        }
        return node;
      } else {
        let name = path.to_godot_string();
        godot_print!("Unable to get node {}", name.to_utf8().as_str());
      }
    }
    None
  }
}

pub trait ConnectTo {
  fn connect_to(self, path: &NodePath, signal: &str, slot: &str) -> bool;
}

impl ConnectTo for Node {
  fn connect_to(self, path: &NodePath, signal: &str, slot: &str) -> bool {
    unsafe {
      if let Some(mut node) = self.get_node(path.new_ref()) {
        // Get the popup if this is a menu button.
        if let Some(button) = node.cast::<MenuButton>() {
          if let Some(popup) = button.get_popup() {
            node = popup.to_node();
          }
        }

        if let Err(err) = node.connect(
          GodotString::from_str(signal),
          Some(self.to_object()),
          GodotString::from_str(slot),
          VariantArray::new(),
          0,
        ) {
          godot_print!("Unable to connect {}: {:?}", slot, err);
        } else {
          return true;
        }
      } else {
        let name = path.to_godot_string();
        godot_print!("Unable to get node {}", name.to_utf8().as_str());
      }
    }
    false
  }
}

pub trait SetShortcut {
  fn set_shortcut(self, id: i64, key: i64, ctrl: bool);
}

impl SetShortcut for PopupMenu {
  fn set_shortcut(mut self, id: i64, key: i64, ctrl: bool) {
    let mut input = InputEventKey::new();
    input.set_control(ctrl);
    input.set_scancode(key);
    unsafe {
      self.set_item_accelerator(self.get_item_index(id), input.get_scancode_with_modifiers());
    }
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
        log_path = Some(GodotString::from_str(path));
      }
    }

    Config {
      log_path: log_path,
      cfg_path: GodotString::from_str("user://settings.cfg"),
      section: GodotString::from_str("main"),
      folder_key: GodotString::from_str("log_folder"),
      avatar_key: GodotString::from_str("avatar"),
    }
  }

  pub fn get_log_folder(&self) -> Option<GodotString> {
    if let Some(folder) = self.get_value(self.folder_key.new_ref()) {
      return Some(folder);
    } else if let Some(folder) = &self.log_path {
      return Some(folder.new_ref());
    }
    None
  }

  pub fn set_log_folder(&self, folder: Option<GodotString>) {
    self.set_value(self.folder_key.new_ref(), folder);
  }

  pub fn get_avatar(&self) -> Option<GodotString> {
    self.get_value(self.avatar_key.new_ref())
  }

  pub fn set_avatar(&self, avatar: Option<GodotString>) {
    self.set_value(self.avatar_key.new_ref(), avatar);
  }

  pub fn _get_notes(&self, avatar: GodotString) -> Option<GodotString> {
    if !avatar.is_empty() {}
    None
  }

  pub fn _set_notes(&self, avatar: GodotString, _notes: Option<GodotString>) {
    if !avatar.is_empty() {}
  }

  fn get_value(&self, key: GodotString) -> Option<GodotString> {
    let mut config = ConfigFile::new();
    if !self.cfg_path.is_empty() && config.load(self.cfg_path.new_ref()).is_ok() {
      if config.has_section_key(self.section.new_ref(), key.new_ref()) {
        let value = config.get_value(self.section.new_ref(), key.new_ref(), Variant::new());
        if !value.is_nil() {
          return Some(value.to_godot_string());
        }
      }
    }
    None
  }

  fn set_value(&self, key: GodotString, value: Option<GodotString>) {
    let mut config = ConfigFile::new();
    let _ = config.load(self.cfg_path.new_ref());
    if let Some(value) = value {
      config.set_value(
        self.section.new_ref(),
        key.new_ref(),
        Variant::from_godot_string(&value),
      );
    } else if config.has_section_key(self.section.new_ref(), key.new_ref()) {
      config.erase_section_key(self.section.new_ref(), key.new_ref());
    }
    let _ = config.save(self.cfg_path.new_ref());
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

  Locale::en_US_POSIX
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

/// Convert a timestamp into a date & time string.
pub fn timestamp_to_view_date(ts: i64) -> String {
  NaiveDateTime::from_timestamp(ts, 0)
    .format("%Y-%m-%d @ %H:%M:%S")
    .to_string()
}

// Convert a SotA log date & time into a timestamp. Since the dates are localized, we don't know
// if day or month come first, so we use the date from the filename, which is always YYYY-MM-DD.
fn log_date_to_timestamp(text: &str, date: &NaiveDate) -> Option<i64> {
  let mut iter = text.split_whitespace();
  let _date = some!(iter.next(), None);
  let time = some!(iter.next(), None);
  let ap = iter.next();

  if iter.next().is_some() {
    return None;
  }

  let mut iter = time.split(':');
  let parts = [
    some!(iter.next(), None),
    some!(iter.next(), None),
    some!(iter.next(), None),
  ];

  if iter.next().is_some() {
    return None;
  }

  // Parse the hour and adjust for PM.
  let hour = {
    let mut hour = ok!(parts[0].parse(), None);
    if let Some(ap) = ap {
      let bytes = ap.as_bytes();
      if bytes.len() > 0 {
        let ch = bytes[0] as char;
        if ch == 'P' || ch == 'p' {
          hour = hour + 12;
          if hour == 24 {
            hour = 0;
          }
        }
      }
    }
    hour
  };

  let minute = ok!(parts[1].parse(), None);
  let second = ok!(parts[2].parse(), None);

  Some(NaiveDateTime::new(date.clone(), NaiveTime::from_hms(hour, minute, second)).timestamp())
}

// Convert a timestamp into a log filename date string.
fn timestamp_to_file_date(ts: i64) -> String {
  NaiveDateTime::from_timestamp(ts, 0)
    .format("%Y-%m-%d")
    .to_string()
}

fn get_log_file_date(path: &Path) -> Option<NaiveDate> {
  let filename = some!(path.file_stem(), None);
  let filename = some!(filename.to_str(), None);
  let pos = some!(filename.rfind('_'), None);
  let text = &filename[pos + 1..];

  if let Ok(date) = NaiveDate::parse_from_str(text, "%Y-%m-%d") {
    return Some(date);
  }

  None
}

fn get_stats_timestamp(line: &str, date: &NaiveDate) -> Option<i64> {
  if line.starts_with('[') {
    if let Some(pos) = line.find(']') {
      if line[pos + 1..].contains(STATS_KEY) {
        return log_date_to_timestamp(&line[1..pos], date);
      }
    }
  }

  None
}

fn get_stats_text<'a>(line: &'a str, ts: i64, date: &NaiveDate) -> Option<&'a str> {
  if let Some(lts) = get_stats_timestamp(line, date) {
    if lts == ts {
      if let Some(pos) = line.rfind(']') {
        return Some(&line[pos + 1..]);
      }
    }
  }

  None
}

const FILENAME_START: &str = "SotAChatLog_";
const STATS_KEY: &str = " AdventurerLevel: ";

pub struct StatsIter<'a> {
  iter: SplitWhitespace<'a>,
}

impl<'a> StatsIter<'a> {
  fn new(text: &str) -> StatsIter {
    StatsIter {
      iter: text.split_whitespace(),
    }
  }
}

impl<'a> Iterator for StatsIter<'a> {
  type Item = (&'a str, &'a str);

  fn next(&mut self) -> Option<Self::Item> {
    while let Some(name) = self.iter.next() {
      if name.ends_with(':') {
        if let Some(value) = self.iter.next() {
          return Some((&name[..name.len() - 1], value));
        }

        break;
      }
    }

    None
  }
}

pub struct StatsData {
  text: String,
}

impl StatsData {
  fn new(text: String) -> StatsData {
    StatsData { text: text }
  }

  pub fn iter<'a>(&'a self) -> StatsIter<'a> {
    StatsIter::new(&self.text)
  }
}

/// Object that reads from SotA chat logs.
pub struct LogData {
  folder: PathBuf,
  pool: RefCell<ThreadPool>,
}

impl LogData {
  pub fn new(folder: GodotString) -> LogData {
    let cpus = num_cpus::get();
    LogData {
      folder: PathBuf::from(folder.to_utf8().as_str()),
      pool: RefCell::new(ThreadPool::new(cpus)),
    }
  }

  /// Get a vector of avatar names.
  pub fn get_avatars(&self) -> Vec<String> {
    let filenames = self.get_log_filenames(None, None);
    let mut name_set = HashSet::<&str>::new();

    for filename in &filenames {
      let filename = &filename[FILENAME_START.len()..];
      if let Some(pos) = filename.rfind('_') {
        name_set.insert(&filename[..pos]);
      }
    }

    let mut avatars = Vec::with_capacity(name_set.len());
    for name in name_set {
      avatars.push(String::from(name));
    }

    avatars.sort_unstable();
    avatars
  }

  /// Get a vector of timestamps where `/stats` was used for the specified avatar.
  pub fn get_stats_timestamps(&self, avatar: &str) -> Vec<i64> {
    let tasks = {
      let filenames = self.get_log_filenames(Some(avatar), None);
      let mut tasks = Vec::new();
      let mut pool = self.pool.borrow_mut();

      // Use all the processing power available to load and parse the log files.
      for filename in filenames {
        let path = self.folder.join(filename.as_str());
        if let Some(date) = get_log_file_date(&path) {
          let task = pool.exec(move |cancel| {
            let mut timestamps = Vec::new();
            if let Ok(text) = fs::read_to_string(&path) {
              for line in text.lines() {
                if cancel() {
                  break;
                }
                if let Some(ts) = get_stats_timestamp(line, &date) {
                  timestamps.push(ts);
                }
              }
            }
            Some(timestamps)
          });
          tasks.push(task);
        }
      }
      tasks
    };

    let mut timestamps = Vec::new();
    for mut task in tasks {
      // Process gtk messages until the task is done.
      while task.current() {
        std::thread::yield_now();
      }

      // Concatenate the results.
      if let Some(mut result) = task.get() {
        timestamps.append(&mut result);
      }
    }

    timestamps.sort_unstable_by(|a, b| b.cmp(a));
    timestamps
  }

  /// Get the stats for the specified avatar and timestamp.
  pub fn get_stats(&self, avatar: &str, ts: i64) -> Option<StatsData> {
    let filenames = self.get_log_filenames(Some(avatar), Some(ts));

    // There will actually only be one file with the specific avatar name and date.
    for filename in filenames {
      let path = self.folder.join(filename.as_str());
      if let Some(date) = get_log_file_date(&path) {
        if let Ok(text) = fs::read_to_string(path) {
          for line in text.lines() {
            if let Some(mut stats) = get_stats_text(line, ts, &date) {
              if stats.len() < 1000 {
                // A Lua script has probably inserted newlines.
                let pos = stats.as_ptr() as usize - text.as_ptr() as usize;
                let mut end = pos + stats.len();

                // Collect subsequent lines that don't have a timestamp.
                for line in text[end..].lines() {
                  end = line.as_ptr() as usize - text.as_ptr() as usize;
                  if line.starts_with('[') {
                    break;
                  }
                }
                stats = &text[pos..end];
              }
              return Some(StatsData::new(String::from(stats)));
            }
          }
        }
      }
    }

    None
  }

  fn get_log_filenames(&self, avatar: Option<&str>, ts: Option<i64>) -> Vec<String> {
    let mut filenames = Vec::new();
    let entries = ok!(self.folder.read_dir(), filenames);

    // The name text is either a specific avatar or a regex wildcard.
    let name = if let Some(avatar) = avatar {
      avatar
    } else {
      ".+"
    };

    // The date text is either a specific date or regex to match the date.
    let date = if let Some(ts) = ts {
      format!("_{}", timestamp_to_file_date(ts))
    } else {
      String::from(r"_\d{4}-\d{2}-\d{2}")
    };

    let regex = ok!(
      Regex::new(&format!("^{}{}{}.txt$", FILENAME_START, name, date)),
      filenames
    );

    for entry in entries {
      if let Ok(entry) = entry {
        if let Ok(filename) = entry.file_name().into_string() {
          if regex.is_match(&filename) {
            filenames.push(filename);
          }
        }
      }
    }

    filenames
  }
}
