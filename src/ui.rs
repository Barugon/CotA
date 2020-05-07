use crate::experience_page::*;
use crate::portals_page::*;
use crate::util::*;
use gdk_pixbuf::{PixbufLoader, PixbufLoaderExt};
use gtk::prelude::*;
use std::{
  cell::RefCell,
  cmp::min,
  collections::HashMap,
  iter::Iterator,
  rc::Rc,
  sync::atomic::{AtomicBool, Ordering},
};

const WIN_WIDTH: i32 = 480;
const WIN_HEIGHT: i32 = 640;
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[derive(Copy, Clone)]
enum Page {
  Stats,
  Portals,
  Experience,
  Count,
}

enum StatOpts<'a> {
  None,
  Resists,
  Filter(&'a str),
}

// A struct to hold all the needed UI elements.
struct Elements {
  closing: Rc<AtomicBool>,
  log_data: RefCell<LogData>,
  settings: RefCell<Settings>,
  status_messages: RefCell<[String; Page::Count as usize]>,
  text_color: Option<gdk::RGBA>,
  window: gtk::ApplicationWindow,
  refresh_menu_item: gtk::MenuItem,
  resists_menu_item: gtk::MenuItem,
  filter_menu_item: gtk::MenuItem,
  reset_menu_item: gtk::MenuItem,
  notebook: gtk::Notebook,
  stats_box: gtk::Box,
  avatars_combo: gtk::ComboBoxText,
  dates_combo: gtk::ComboBoxText,
  notes_button: gtk::Button,
  tree_view: gtk::TreeView,
  status_bar: gtk::Statusbar,
  portals_page: PortalsPage,
  experience_page: ExperiencePage,
}

impl Elements {
  fn enable_view_items(&self, enable: bool) {
    self.resists_menu_item.set_sensitive(enable);
    self.filter_menu_item.set_sensitive(enable);
    self.reset_menu_item.set_sensitive(enable);
  }

  fn get_log_folder(&self) -> String {
    self.settings.borrow().get_log_folder()
  }

  fn set_log_folder(&self, folder: &str) -> bool {
    if self.settings.borrow_mut().set_log_folder(folder) {
      *self.log_data.borrow_mut() = LogData::new(String::from(folder), Rc::clone(&self.closing));
      return true;
    }

    false
  }

  fn select_log_folder(&self) {
    let dlg = gtk::FileChooserDialog::new(
      Some(t!("Select SotA Log Folder")),
      Some(&self.window),
      gtk::FileChooserAction::SelectFolder,
    );
    dlg.set_filename(&self.get_log_folder());
    dlg.add_buttons(&[
      (t!("Ok"), gtk::ResponseType::Ok.into()),
      (t!("Cancel"), gtk::ResponseType::Cancel.into()),
    ]);

    if gtk::ResponseType::from(dlg.run()) == gtk::ResponseType::Ok {
      if let Some(folder) = dlg.get_filename() {
        if let Some(folder) = folder.to_str() {
          if self.set_log_folder(folder) {
            // Hide the dialog before populating the avatars.
            dlg.hide();
            self.populate_avatars("");
          }
        }
      }
    }

    dlg.destroy();
  }

  fn get_filter(&self) -> Option<glib::GString> {
    // Create the dialog with OK and Cancel buttons.
    let dlg = gtk::Dialog::new();
    dlg.set_default_size(10, 10);
    dlg.set_title(t!("Filter"));
    dlg.set_transient_for(Some(&self.window));
    dlg.add_buttons(&[
      (t!("Ok"), gtk::ResponseType::Ok.into()),
      (t!("Cancel"), gtk::ResponseType::Cancel.into()),
    ]);

    // Add the text entry field.
    let entry = gtk::Entry::new();
    entry.set_visible(true);
    entry.connect_activate(func!(dlg => move |_| {
      dlg.response(gtk::ResponseType::Ok.into());
    }));

    let content_area = dlg.get_content_area();
    content_area.set_margins(3, 3, 3, 3);
    content_area.set_spacing(3);
    content_area.pack_start(&entry, false, true, 0);

    let filter = if gtk::ResponseType::from(dlg.run()) == gtk::ResponseType::Ok {
      entry.get_text()
    } else {
      None
    };

    dlg.destroy();
    filter
  }

  fn about(&self) {
    let icon = some!(self.window.get_icon());
    let authors: Vec<&str> = env!("CARGO_PKG_AUTHORS").split(':').collect();
    let dlg = gtk::AboutDialog::new();
    dlg.set_transient_for(Some(&self.window));
    dlg.set_authors(&authors);
    dlg.set_comments(Some(DESCRIPTION));
    dlg.set_logo(Some(&icon));
    dlg.set_program_name("CotA");
    dlg.set_version(Some(env!("CARGO_PKG_VERSION")));
    dlg.set_website(Some("https://github.com/Barugon/CotA"));
    dlg.run();
    dlg.destroy();
  }

  fn set_stored_avatar(&self, avatar: &str) -> bool {
    self.settings.borrow_mut().set_avatar(&avatar)
  }

  fn get_stored_avatar(&self) -> String {
    self.settings.borrow().get_avatar()
  }

  fn get_avatars(&self) -> Vec<String> {
    self.log_data.borrow().get_avatars()
  }

  fn populate_avatars(&self, default_avatar: &str) {
    self.avatars_combo.remove_all();
    self.dates_combo.remove_all();
    self.reset_stats();
    self.enable_view_items(false);
    self.notes_button.set_sensitive(false);

    let mut avatars = self.get_avatars();
    if avatars.is_empty() {
      self.set_status_message(Page::Stats as usize, Some(t!("No avatars found")));
      return;
    }

    avatars.sort_unstable();
    for avatar in avatars {
      let text: &str = &avatar;
      self.avatars_combo.append(Some(text), text);
    }

    if !default_avatar.is_empty() {
      self.avatars_combo.set_active_id(Some(default_avatar));
    }

    if !self.avatars_combo.get_active().is_some() {
      self.avatars_combo.set_active(Some(0));
    }
  }

  fn get_stats_timestamps(&self, avatar: &str) -> Vec<i64> {
    let mut timestamps = {
      // Set a wait cursor on the main window.
      let _wc = WaitCursor::new(&self.window);

      // Disable the stats_box and all child widgets.
      let _ws = WidgetSensitivity::new(&self.stats_box, false);

      // Get the timestamps.
      self.log_data.borrow().get_stats_timestamps(avatar)
    };

    // Sort the timestamps.
    timestamps.sort_unstable_by(|a, b| b.cmp(a));
    timestamps
  }

  fn populate_dates(&self) {
    self.dates_combo.remove_all();
    self.reset_stats();
    self.enable_view_items(false);

    if let Some(avatar) = self.avatars_combo.get_active_text() {
      self.set_status_message(Page::Stats as usize, Some(&String::new()));
      self.set_stored_avatar(&avatar);

      let timestamps = self.get_stats_timestamps(&avatar);
      if self.closing.load(Ordering::Relaxed) {
        return;
      }

      self.notes_button.set_sensitive(true);
      if timestamps.is_empty() {
        self.set_status_message(
          Page::Stats as usize,
          Some(&t!("No stats found for {}").replace("{}", &avatar)),
        );
        return;
      }

      // Sort the timestamps in reverse order so that the most recent is first.
      for ts in timestamps {
        let date = timestamp_to_view_date(ts);
        self.dates_combo.append(None, &date);
      }

      self.dates_combo.set_active(Some(0));
    } else {
      self.notes_button.set_sensitive(false);
    }
  }

  fn get_notes(&self, avatar: &str) -> String {
    self.settings.borrow().get_notes(avatar)
  }

  fn set_notes(&self, avatar: &str, text: &str) -> bool {
    self.settings.borrow_mut().set_notes(avatar, text)
  }

  fn modify_notes(&self) {
    let avatar = some!(self.avatars_combo.get_active_text());

    // Create the dialog with OK and Cancel buttons.
    let dlg = gtk::Dialog::new();
    dlg.set_default_size(400, 300);
    dlg.set_title(&t!("Notes for {}").replace("{}", &avatar));
    dlg.set_transient_for(Some(&self.window));
    dlg.add_buttons(&[
      (t!("Ok"), gtk::ResponseType::Ok.into()),
      (t!("Cancel"), gtk::ResponseType::Cancel.into()),
    ]);

    // Add the text entry field.
    let text_view = gtk::TextView::new();
    text_view.set_visible(true);

    // Get previously stored notes.
    let text = self.get_notes(&avatar);
    if !text.is_empty() {
      text_view.set_text(&text);
    }

    let frame = gtk::Frame::new(None);
    frame.add(&text_view);
    frame.set_visible(true);

    let content_area = dlg.get_content_area();
    content_area.set_margins(3, 3, 3, 3);
    content_area.set_spacing(5);
    content_area.pack_start(&frame, true, true, 0);

    // Run the dialog and get the result.
    if gtk::ResponseType::from(dlg.run()) == gtk::ResponseType::Ok {
      if let Some(text) = text_view.get_text() {
        self.set_notes(&avatar, &text);
      }
    }

    dlg.destroy();
  }

  fn get_stats(&self, avatar: &str, ts: i64) -> Option<Stats> {
    self.log_data.borrow().get_stats(avatar, ts)
  }

  fn reset_stats(&self) {
    if let Some(model) = self.tree_view.get_model() {
      if !model.get_iter_first().is_some() {
        return;
      }
    }

    const LIST_COLS: [glib::Type; 4] = [
      glib::Type::String,
      glib::Type::String,
      glib::Type::String,
      glib::Type::F64,
    ];
    self
      .tree_view
      .set_model(Some(&gtk::ListStore::new(&LIST_COLS)));
  }

  fn populate_stats(&self, opts: StatOpts) {
    self.reset_stats();

    let avatar = some!(self.avatars_combo.get_active_text());
    let date = some!(self.dates_combo.get_active_text());
    let ts = some!(view_date_to_timestamp(&date));
    let model = some!(self.tree_view.get_model());
    let list_store = ok!(model.dynamic_cast::<gtk::ListStore>());
    let locale = get_locale();

    if let Some(stats) = self.get_stats(&avatar, ts) {
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
          let resist_stats: HashMap<&str, (Resists, f64)> = RESIST_STATS.iter().cloned().collect();
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

          let markup = if let Some(color) = self.text_color {
            [
              String::from("<span foreground='#7F7F00'><b>{}</b></span>"),
              format!("<span foreground='{}'>{{}}</span>", html_color(&color, 0.5)),
            ]
          } else {
            [String::from("{}"), String::from("{}")]
          };

          // Format the output.
          for (pos, key) in RESIST_KEYS.iter().enumerate() {
            if let Some(value) = resist_values.get(key) {
              let name = RESIST_NAMES[pos];
              let name_sort = glib::Value::from(name);
              let value_sort = glib::Value::from(&value);
              let name = glib::Value::from(&markup[0].replacen("{}", name, 1));
              let value = value.to_display_string(&locale);
              let value = glib::Value::from(&markup[1].replacen("{}", &value, 1));
              list_store.insert_with_values(
                None,
                &[0, 1, 2, 3],
                &[&name, &value, &name_sort, &value_sort],
              );
            }
          }

          self.set_status_message(
            Page::Stats as usize,
            Some(&t!("Showing effective resists from {}").replacen("{}", &date, 1)),
          );
        }
        _ => {
          const ORDER: [(&str, i32); 5] = [
            ("VirtueCourage", -1),
            ("VirtueLove", -1),
            ("VirtueTruth", -1),
            ("AdventurerLevel", 0),
            ("ProducerLevel", 1),
          ];
          let mut bins: [Vec<(&str, &str)>; 3] = [Vec::new(), Vec::new(), Vec::new()];
          let order: HashMap<&str, i32> = ORDER.iter().cloned().collect();

          for (name, value) in stats.iter() {
            let checked = if let StatOpts::Filter(filter) = opts {
              // Check if the name contains the filter string.
              if !ascii_contains_ignore_case(name.as_bytes(), filter.as_bytes()) {
                continue;
              }
              true
            } else {
              false
            };

            // Sort the stat into the appropriate bin.
            if let Some(bin) = order.get(name) {
              let bin = *bin;
              if bin >= 0 {
                bins[min(bin as usize, bins.len())].push((name, value));
              } else if checked {
                // Always display values that have passed the filter check.
                bins[2].push((name, value));
              }
            } else {
              bins[2].push((name, value));
            }
          }

          // Generate the HTML markup.
          let markup = if let Some(color) = self.text_color {
            [
              format!("<span foreground='{}'>{{}}</span>", html_color(&color, 1.0)),
              format!("<span foreground='{}'>{{}}</span>", html_color(&color, 0.7)),
              format!("<span foreground='{}'>{{}}</span>", html_color(&color, 0.4)),
            ]
          } else {
            [String::from("{}"), String::from("{}"), String::from("{}")]
          };

          // Insert the stat items into the ListStore.
          for index in 0..bins.len() {
            for (name, value) in &bins[index] {
              if let Ok(value) = value.replacen(',', ".", 1).parse::<f64>() {
                let name_sort = glib::Value::from(name);
                let value_sort = glib::Value::from(&value);
                let name = glib::Value::from(&markup[index].replacen("{}", name, 1));
                let value = value.to_display_string(&locale);
                let value = glib::Value::from(&markup[index].replacen("{}", &value, 1));
                list_store.insert_with_values(
                  None,
                  &[0, 1, 2, 3],
                  &[&name, &value, &name_sort, &value_sort],
                );
              }
            }
          }

          self.set_status_message(
            Page::Stats as usize,
            Some(&t!("Showing stats from {}").replacen("{}", &date, 1)),
          );
        }
      }

      self.tree_view.set_search_column(2);
      self.enable_view_items(true);
    } else {
      self.set_status_message(
        Page::Stats as usize,
        Some(&t!("No stats found on {}").replacen("{}", &date, 1)),
      );
    }
  }

  fn is_current_page(&self, page: usize) -> bool {
    if let Some(cur) = self.notebook.get_current_page() {
      return cur as usize == page;
    }

    false
  }

  fn set_status_message(&self, page: usize, text: Option<&str>) {
    if let Some(text) = text {
      if self.is_current_page(page) {
        self.status_bar.remove_all(0);
        self.status_bar.push(0, text);
      }
      self.status_messages.borrow_mut()[page] = String::from(text);
    } else {
      // Set from a previously stored status message.
      let text = &self.status_messages.borrow()[page];
      self.status_bar.remove_all(0);
      self.status_bar.push(0, text);
    }
  }
}

/// Initialize and layout the application's GUI.
pub fn layout(app: &gtk::Application) {
  let closing = Rc::new(AtomicBool::new(false));
  let settings = Settings::new();
  let log_data = LogData::new(settings.get_log_folder(), Rc::clone(&closing));

  // Initialize the application window.
  let window = gtk::ApplicationWindow::new(app);
  window.set_title(DESCRIPTION);
  window.set_resizable(false);
  window.set_default_size(WIN_WIDTH, WIN_HEIGHT);
  window.connect_delete_event(func!(closing => move |win, _| {
    closing.store(true, Ordering::Relaxed);
    win.destroy();
    Inhibit(false)
  }));

  // Set the window's icon.
  let icon_data = include_bytes!("../res/cota.png");
  let pixbuf_loader = PixbufLoader::new();
  if pixbuf_loader.write(icon_data).is_ok() {
    if let Some(icon) = pixbuf_loader.get_pixbuf() {
      window.set_icon(Some(&icon));
    }
  }
  let _ = pixbuf_loader.close();

  // Initialize the elements struct.
  let elements = Rc::new(Elements {
    closing: closing,
    log_data: RefCell::new(log_data),
    settings: RefCell::new(settings),
    status_messages: RefCell::new(Default::default()),
    text_color: window.get_text_color(),
    window: window.clone(),
    refresh_menu_item: gtk::MenuItem::new_with_mnemonic(t!("_Refresh stats")),
    resists_menu_item: gtk::MenuItem::new_with_mnemonic(t!("_Effective resists")),
    filter_menu_item: gtk::MenuItem::new_with_mnemonic(t!("_Filter stats...")),
    reset_menu_item: gtk::MenuItem::new_with_mnemonic(t!("Reset")),
    notebook: gtk::Notebook::new(),
    stats_box: gtk::Box::new(gtk::Orientation::Vertical, 0),
    avatars_combo: gtk::ComboBoxText::new(),
    dates_combo: gtk::ComboBoxText::new(),
    notes_button: gtk::Button::new_with_label(t!("Notes")),
    tree_view: gtk::TreeView::new(),
    status_bar: gtk::Statusbar::new(),
    portals_page: PortalsPage::new(),
    experience_page: ExperiencePage::new(),
  });

  let accel_group = gtk::AccelGroup::new();
  window.add_accel_group(&accel_group);

  elements
    .avatars_combo
    .connect_changed(func!(elements => move |_| elements.populate_dates()));
  elements
    .dates_combo
    .connect_changed(func!(elements => move |_| elements.populate_stats(StatOpts::None)));
  elements
    .notes_button
    .connect_clicked(func!(elements => move |_| elements.modify_notes()));

  let log_folder_menu_item = gtk::MenuItem::new_with_mnemonic(t!("_Log Folder..."));
  log_folder_menu_item.connect_activate(func!(elements => move |_| elements.select_log_folder()));

  let quit_menu_item = gtk::MenuItem::new_with_mnemonic(t!("_Quit"));
  quit_menu_item.connect_activate(func!(elements => move |_| elements.window.destroy()));

  let (key, modifier) = gtk::accelerator_parse("<Primary>Q");
  quit_menu_item.add_accelerator(
    "activate",
    &accel_group,
    key,
    modifier,
    gtk::AccelFlags::VISIBLE,
  );

  let file_menu = gtk::Menu::new();
  file_menu.append(&log_folder_menu_item);
  file_menu.append(&gtk::SeparatorMenuItem::new());
  file_menu.append(&quit_menu_item);

  let file_menu_item = gtk::MenuItem::new_with_mnemonic(t!("_File"));
  file_menu_item.set_submenu(Some(&file_menu));

  elements
    .refresh_menu_item
    .connect_activate(func!(elements => move |_| if let Some(avatar) =
      elements.avatars_combo.get_active_text()
    {
      if elements.stats_box.is_sensitive() {
        elements.populate_avatars(&avatar);
      }
    }));

  let (key, modifier) = gtk::accelerator_parse("F5");
  elements.refresh_menu_item.add_accelerator(
    "activate",
    &accel_group,
    key,
    modifier,
    gtk::AccelFlags::VISIBLE,
  );

  elements
    .resists_menu_item
    .connect_activate(func!(elements => move |_| elements.populate_stats(StatOpts::Resists)));

  let (key, modifier) = gtk::accelerator_parse("<Primary>R");
  elements.resists_menu_item.add_accelerator(
    "activate",
    &accel_group,
    key,
    modifier,
    gtk::AccelFlags::VISIBLE,
  );

  elements
    .filter_menu_item
    .connect_activate(func!(elements => move |_| if let Some(filter) =
      elements.get_filter()
    {
      elements.populate_stats(StatOpts::Filter(&filter));
    }));

  let (key, modifier) = gtk::accelerator_parse("<Primary>F");
  elements.filter_menu_item.add_accelerator(
    "activate",
    &accel_group,
    key,
    modifier,
    gtk::AccelFlags::VISIBLE,
  );

  elements
    .reset_menu_item
    .connect_activate(func!(elements => move |_| elements.populate_stats(StatOpts::None)));
  elements.reset_menu_item.add_accelerator(
    "activate",
    &accel_group,
    gdk::enums::key::Escape,
    gdk::ModifierType::empty(),
    gtk::AccelFlags::VISIBLE,
  );

  let view_menu = gtk::Menu::new();
  view_menu.append(&elements.refresh_menu_item);
  view_menu.append(&gtk::SeparatorMenuItem::new());
  view_menu.append(&elements.resists_menu_item);
  view_menu.append(&elements.filter_menu_item);
  view_menu.append(&elements.reset_menu_item);

  let view_menu_item = gtk::MenuItem::new_with_mnemonic(t!("_View"));
  view_menu_item.set_submenu(Some(&view_menu));

  let about_menu_item = gtk::MenuItem::new_with_mnemonic(t!("_About"));
  about_menu_item.connect_activate(func!(elements => move |_| elements.about()));

  let help_menu = gtk::Menu::new();
  help_menu.append(&about_menu_item);

  let help_menu_item = gtk::MenuItem::new_with_mnemonic(t!("_Help"));
  help_menu_item.set_submenu(Some(&help_menu));

  let cell = gtk::CellRendererText::new();
  let name_column = gtk::TreeViewColumn::new();
  name_column.set_title(t!("Name"));
  name_column.pack_start(&cell, true);
  name_column.add_attribute(&cell, "markup", 0);
  name_column.set_sort_column_id(2);

  let value_column = gtk::TreeViewColumn::new();
  value_column.set_title(t!("Value"));
  value_column.pack_start(&cell, true);
  value_column.add_attribute(&cell, "markup", 1);
  value_column.connect_clicked(|_| {});
  value_column.set_sort_column_id(3);

  let tree_view = elements.tree_view.clone();
  tree_view.append_column(&name_column);
  tree_view.append_column(&value_column);

  let tool_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
  let label = gtk::Label::new(Some(t!("Avatar:")));
  tool_box.set_margins(5, 3, 5, 3);
  tool_box.pack_start(&label, false, true, 0);
  tool_box.pack_start(&elements.avatars_combo, true, true, 0);
  tool_box.pack_start(&elements.dates_combo, true, true, 0);
  tool_box.pack_start(&elements.notes_button, false, true, 0);

  let scrolled_window =
    gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
  scrolled_window.add(&tree_view);

  elements.stats_box.pack_start(&tool_box, false, true, 0);
  elements
    .stats_box
    .pack_start(&scrolled_window, true, true, 0);

  let stats_label = gtk::Label::new(Some(t!("Stats")));
  let portals_label = gtk::Label::new(Some(t!("Portals")));
  let xp_label = gtk::Label::new(Some(t!("Experience")));
  let notebook = elements.notebook.clone();
  notebook.insert_page(
    &elements.stats_box,
    Some(&stats_label),
    Some(Page::Stats as u32),
  );
  notebook.insert_page(
    &elements.portals_page.page_box,
    Some(&portals_label),
    Some(Page::Portals as u32),
  );
  notebook.insert_page(
    &elements.experience_page.page_box,
    Some(&xp_label),
    Some(Page::Experience as u32),
  );
  notebook.connect_switch_page(func!(elements => move |_, _, page| {
    if page == Page::Stats as u32 {
      elements.refresh_menu_item.set_sensitive(true);
      if let Some(model) = elements.tree_view.get_model() {
        if let Some(_) = model.get_iter_first() {
          elements.enable_view_items(true);
        }
      }
    } else {
      elements.refresh_menu_item.set_sensitive(false);
      elements.enable_view_items(false);

      if page == Page::Portals as u32 {
        elements.portals_page.update();
      }
    }

    elements.set_status_message(page as usize, None);
  }));

  let menu_bar = gtk::MenuBar::new();
  menu_bar.append(&file_menu_item);
  menu_bar.append(&view_menu_item);
  menu_bar.append(&help_menu_item);

  let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
  main_box.pack_start(&menu_bar, false, false, 0);
  main_box.pack_start(&elements.notebook, true, true, 0);
  main_box.pack_start(&elements.status_bar, false, true, 0);

  // Timer to update the portals page.
  let weak = Rc::downgrade(&elements);
  gtk::timeout_add(1000, move || {
    if let Some(elements) = weak.upgrade() {
      if elements.notebook.get_current_page() == Some(Page::Portals as u32) {
        elements.portals_page.update();
      }
      Continue(true)
    } else {
      Continue(false)
    }
  });

  window.add(&main_box);
  window.show_all();

  // Populate the avatars after the window is visible.
  elements.populate_avatars(&elements.get_stored_avatar());
}
