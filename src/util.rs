use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use num_cpus;
use num_format::Locale;
use regex::Regex;
use std::{
  cell::RefCell, collections::HashSet, fs, path::Path, path::PathBuf, str::SplitWhitespace,
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

pub fn _ascii_contains_ignore_case(container: &[u8], pattern: &[u8]) -> bool {
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

pub fn get_locale() -> Locale {
  let name = gdnative::OS::godot_singleton().get_locale();
  let name = name.to_utf8().as_str().replace('_', "-");
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
  pub fn new(folder: PathBuf) -> LogData {
    let cpus = num_cpus::get();
    LogData {
      folder: folder,
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
