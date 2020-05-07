use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use gdk::WindowExt;
use gtk::prelude::*;
use num_cpus;
use num_format::Locale;
use regex::Regex;
use serde_json::{json, Value};
use std::{
  cell::RefCell,
  collections::HashSet,
  env, fs,
  path::{Path, PathBuf},
  rc::Rc,
  str::SplitWhitespace,
  sync::atomic::{AtomicBool, Ordering},
};
use thread_pool::*;

#[macro_export]
macro_rules! func {
  (@param _) => ( _ );
  (@param $x:ident) => ( $x );
  ($($n:ident),+ => move || $body:expr) => (
    {
      $( let $n = $n.clone(); )+
      move || $body
    }
  );
  ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
    {
      $( let $n = $n.clone(); )+
      move |$(func!(@param $p),)+| $body
    }
  );
}

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
        println!("{}", err);
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
        println!("{}", err);
      }
      return $ret;
    }
  }};
}

#[macro_export]
macro_rules! t {
  ($txt:expr) => {
    Translate::new($txt).get()
  };
}

pub struct Translate<'a> {
  opt: Option<glib::GString>,
  txt: &'a str,
}

impl Translate<'_> {
  pub fn new(txt: &str) -> Translate {
    Translate {
      opt: glib::dgettext(None, txt),
      txt: txt,
    }
  }

  pub fn get(&self) -> &str {
    if let Some(txt) = &self.opt {
      return txt;
    }

    self.txt
  }
}

pub fn get_locale() -> Locale {
  if let Some(language) = gtk::get_default_language() {
    let name = language.to_string();
    let mut iter = Locale::available_names().iter();

    // Search for an exact match.
    if let Some(name) = iter.find(|n| ascii_equals_ignore_case(n.as_bytes(), name.as_bytes())) {
      if let Ok(locale) = Locale::from_name(name) {
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
  }

  Locale::en
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

pub fn ascii_equals_ignore_case(left: &[u8], right: &[u8]) -> bool {
  left.len() == right.len() && ascii_starts_with_ignore_case(left, right)
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

/// Expanded functionality for GTK Widgets.
pub trait ExpWidget {
  /// Set the left, top, right and bottom margins for a widget.
  fn set_margins(&self, left: i32, top: i32, right: i32, bottom: i32);

  /// Get the text color for a widget.
  fn get_text_color(&self) -> Option<gdk::RGBA>;
}

impl<T: gtk::WidgetExt> ExpWidget for T {
  fn set_margins(&self, start: i32, top: i32, end: i32, bottom: i32) {
    self.set_margin_start(start);
    self.set_margin_top(top);
    self.set_margin_end(end);
    self.set_margin_bottom(bottom);
  }

  fn get_text_color(&self) -> Option<gdk::RGBA> {
    let context = self.get_style_context();
    match context.lookup_color("text_color") {
      Some(color) => Some(color),
      None => context.lookup_color("theme_text_color"),
    }
  }
}

/// Expanded functionality for GTK TextView.
pub trait ExpTextView {
  fn set_text(&self, text: &str);
  fn get_text(&self) -> Option<glib::GString>;
}

impl<T: gtk::TextViewExt> ExpTextView for T {
  fn set_text(&self, text: &str) {
    if let Some(buffer) = self.get_buffer() {
      buffer.set_text(&text);
    }
  }

  fn get_text(&self) -> Option<glib::GString> {
    if let Some(buffer) = self.get_buffer() {
      buffer.get_text(&buffer.get_start_iter(), &buffer.get_end_iter(), false)
    } else {
      None
    }
  }
}

/// A simple wait cursor object for lengthy operations.
pub struct WaitCursor {
  win: Option<gdk::Window>,
}

impl WaitCursor {
  pub fn new<T: gtk::WidgetExt>(widget: &T) -> WaitCursor {
    let win = widget.get_window();
    if let Some(win) = &win {
      let cursor = gdk::Cursor::new_for_display(&win.get_display(), gdk::CursorType::Watch);
      win.set_cursor(Some(&cursor));
    }

    WaitCursor { win: win }
  }
}

impl Drop for WaitCursor {
  fn drop(&mut self) {
    if let Some(win) = self.win.take() {
      // Reset the cursor to the default.
      win.set_cursor(None);
    }
  }
}

/// Helper for setting a widgets sensitivity.
pub struct WidgetSensitivity<'a, T: gtk::WidgetExt> {
  widget: &'a T,
  sensitive: bool,
}

impl<'a, T: gtk::WidgetExt> WidgetSensitivity<'a, T> {
  pub fn new(widget: &T, sensitive: bool) -> WidgetSensitivity<T> {
    let current = widget.get_sensitive();
    widget.set_sensitive(sensitive);

    WidgetSensitivity {
      widget: widget,
      sensitive: current,
    }
  }
}

impl<'a, T: gtk::WidgetExt> Drop for WidgetSensitivity<'a, T> {
  fn drop(&mut self) {
    self.widget.set_sensitive(self.sensitive);
  }
}

trait Clmp {
  /// Clamp a value between min and max.
  /// > Note: this is named `clmp` because `clamp` is part of the future standard.
  fn clmp(self, min: Self, max: Self) -> Self;
}

impl Clmp for f64 {
  fn clmp(self, min: Self, max: Self) -> Self {
    self.max(min).min(max)
  }
}

/// Convert an RGBA color to HTML notation.
pub fn html_color(color: &gdk::RGBA, opacity: f64) -> String {
  format!(
    "#{:02X}{:02X}{:02X}{:02X}",
    (color.red.clmp(0.0, 1.0) * 255.0).round() as u8,
    (color.green.clmp(0.0, 1.0) * 255.0).round() as u8,
    (color.blue.clmp(0.0, 1.0) * 255.0).round() as u8,
    (color.alpha.clmp(0.0, 1.0) * opacity.clmp(0.0, 1.0) * 255.0).round() as u8
  )
}

/// Get the current lunar phase as f64.
pub fn get_lunar_phase() -> f64 {
  // Get the elapsed time since the lunar rift epoch.
  let dur = Utc::now() - Utc.ymd(1997, 9, 2).and_hms(0, 0, 0);

  // Calculate the lunar phase from the duration. Each phase is 525 seconds and there are 8 phases, for a total of 4200
  // seconds per lunar cycle.
  return (dur.num_seconds() % 4200) as f64 / 525.0;
}

/// Get the current Lost Vale countdown (in minutes) as f64.
pub fn get_lost_vale_countdown() -> f64 {
  // Get the elapsed time since 2018/02/23 13:00:00 UTC (first sighting).
  let dur = Utc::now() - Utc.ymd(2018, 2, 23).and_hms(13, 0, 0);

  // Calculate the time window using the original 28 hour duration.
  const HSECS: i64 = 60 * 60;
  let win = dur.num_seconds() % (28 * HSECS);

  // Get the 11-11-6 hour segment within the time window (new as of R57).
  let seg = win % (11 * HSECS);

  if seg < HSECS {
    // Lost vale is currently open.
    -(HSECS - seg) as f64 / 60.0
  } else if win < (22 * HSECS) {
    // First two 11 hour segments.
    (11 * HSECS - seg) as f64 / 60.0
  } else {
    // Last 6 hour segment.
    (6 * HSECS - seg) as f64 / 60.0
  }
}

const LOG_FOLDER: &str = "Log Folder";
const AVATAR: &str = "Avatar";
const NOTES: &str = "Notes";

/// Application settings object.
pub struct Settings {
  path: PathBuf,
  json: Value,
}

impl Settings {
  pub fn new() -> Self {
    // Get the application's path.
    if let Some(path) = env::args().next() {
      let path = Path::new(&path).with_extension("json");

      // Attempt to read the settings file.
      if let Ok(text) = fs::read_to_string(path.as_path()) {
        if let Ok(json) = serde_json::from_str::<Value>(&text) {
          if json.is_object() {
            return Settings {
              path: path,
              json: json,
            };
          }
        }
      }

      return Settings {
        path: path,
        json: json!({}),
      };
    }

    Settings {
      path: PathBuf::new(),
      json: json!({}),
    }
  }

  /// Get the log folder path.
  pub fn get_log_folder(&self) -> String {
    let folder = self.get(LOG_FOLDER);
    if folder.is_empty() {
      // Construct the default log folder path.
      if let Some(mut path) = glib::get_home_dir() {
        path.push(if cfg!(windows) {
          r"AppData\Roaming\Portalarium\Shroud of the Avatar\ChatLogs"
        } else {
          // This should be the same on Mac and Linux.
          ".config/Portalarium/Shroud of the Avatar/ChatLogs"
        });

        if let Some(folder) = path.to_str() {
          return String::from(folder);
        }
      }
    }

    folder
  }

  /// Set the log folder path.
  ///
  /// ### Returns
  /// `true` if different from what's currently stored.
  pub fn set_log_folder(&mut self, folder: &str) -> bool {
    self.set_and_store(LOG_FOLDER, folder)
  }

  /// Get the current avatar name.
  pub fn get_avatar(&self) -> String {
    self.get(AVATAR)
  }

  /// Set the current avatar name.
  ///
  /// ### Returns
  /// `true` if different from what's currently stored.
  pub fn set_avatar(&mut self, avatar: &str) -> bool {
    self.set_and_store(AVATAR, avatar)
  }

  /// Get the notes for the specified avatar.
  pub fn get_notes(&self, avatar: &str) -> String {
    if !avatar.is_empty() {
      return self.get(&format!("{} {}", avatar, NOTES));
    }

    String::new()
  }

  /// Set the notes for the specified avatar.
  ///
  /// ### Returns
  /// `true` if different from what's currently stored.
  pub fn set_notes(&mut self, avatar: &str, notes: &str) -> bool {
    if !avatar.is_empty() {
      return self.set_and_store(&format!("{} {}", avatar, NOTES), notes);
    }

    false
  }

  fn get(&self, key: &str) -> String {
    if let Some(val) = self.json.get(key) {
      if let Some(val) = val.as_str() {
        return String::from(val);
      }
    }

    String::new()
  }

  fn set_and_store(&mut self, key: &str, text: &str) -> bool {
    // Compare to the currently stored value.
    if let Some(val) = self.json.get(key) {
      if let Some(val) = val.as_str() {
        if val == text {
          return false;
        }
      }
    }

    if let Some(obj) = self.json.as_object_mut() {
      if text.is_empty() {
        // Remove the value.
        if obj.remove(key).is_none() {
          return false;
        }
      } else {
        // Set the value.
        let key = String::from(key);
        let value = Value::from(String::from(text));
        obj.insert(key, value);
      }
    }

    // Write out the JSON.
    if self.path.file_name().is_some() {
      let _ = fs::write(self.path.as_path(), self.json.to_string().as_str());
    }

    true
  }
}

/// Convert a timestamp into a date & time string.
pub fn timestamp_to_view_date(ts: i64) -> String {
  NaiveDateTime::from_timestamp(ts, 0)
    .format("%Y-%m-%d @ %H:%M:%S")
    .to_string()
}

/// Convert a date & time string to a timestamp.
pub fn view_date_to_timestamp(date: &str) -> Option<i64> {
  if let Ok(date) = NaiveDateTime::parse_from_str(date, "%Y-%m-%d @ %H:%M:%S") {
    Some(date.timestamp())
  } else {
    None
  }
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

pub struct Stats {
  text: String,
}

impl Stats {
  fn new(text: String) -> Stats {
    Stats { text: text }
  }

  pub fn iter<'a>(&'a self) -> StatsIter<'a> {
    StatsIter::new(&self.text)
  }
}

/// Object that reads from SotA chat logs.
pub struct LogData {
  folder: String,
  pool: RefCell<ThreadPool>,
  closing: Rc<AtomicBool>,
}

impl LogData {
  pub fn new(folder: String, closing: Rc<AtomicBool>) -> LogData {
    LogData {
      folder: folder,
      pool: RefCell::new(ThreadPool::new(num_cpus::get())),
      closing: closing,
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
        let path = Path::new(&self.folder).join(filename);
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
        gtk::main_iteration();

        // Cancel the task if the app is closing.
        if self.closing.load(Ordering::Relaxed) {
          task.cancel();
          break;
        }
      }

      // Concatenate the results.
      if let Some(mut result) = task.get() {
        timestamps.append(&mut result);
      }
    }

    timestamps
  }

  /// Get the stats for the specified avatar and timestamp.
  pub fn get_stats(&self, avatar: &str, ts: i64) -> Option<Stats> {
    let filenames = self.get_log_filenames(Some(avatar), Some(ts));
    let folder = Path::new(&self.folder);

    // There will actually only be one file with the specific avatar name and date.
    for filename in filenames {
      let path = folder.join(filename);
      if let Some(date) = get_log_file_date(&path) {
        if let Ok(text) = fs::read_to_string(&path) {
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
              return Some(Stats::new(String::from(stats)));
            }
          }
        }
      }
    }

    None
  }

  fn get_log_filenames(&self, avatar: Option<&str>, ts: Option<i64>) -> Vec<String> {
    let mut filenames = Vec::new();
    let entries = ok!(Path::new(&self.folder).read_dir(), filenames);

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

/// Get the amount of experience needed in order to raise a skill to the specified level.
pub fn exp_delta(cur_lvl: usize, tgt_lvl: usize) -> i64 {
  assert!(cur_lvl >= 1 && cur_lvl <= 200 && tgt_lvl >= 1 && tgt_lvl <= 200);
  const EXP_VALUES: [i64; 200] = include!("../res/exp_values");
  EXP_VALUES[tgt_lvl - 1] - EXP_VALUES[cur_lvl - 1]
}
