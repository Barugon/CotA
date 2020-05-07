use crate::util::*;
use gtk::prelude::*;
use num_format::{Locale, ToFormattedString};
use std::rc::Rc;

pub struct ExperiencePage {
  pub page_box: gtk::Box,
}

enum ExpType {
  Adventurer,
  Producer,
}

impl ExperiencePage {
  pub fn new() -> ExperiencePage {
    let locale = get_locale();

    let adv_group = ExperienceGroup::new(ExpType::Adventurer, locale);
    adv_group.group_box.set_margins(20, 20, 20, 20);

    let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
    separator.set_size_request(1, 2);
    separator.set_margins(5, 0, 5, 0);

    let craft_group = ExperienceGroup::new(ExpType::Producer, locale);
    craft_group.group_box.set_margins(20, 20, 20, 20);

    let page_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    page_box.pack_start(&adv_group.group_box, true, true, 0);
    page_box.pack_start(&separator, false, true, 0);
    page_box.pack_start(&craft_group.group_box, true, true, 0);

    ExperiencePage { page_box: page_box }
  }
}

struct ExperienceGroup {
  group_box: gtk::Box,
  tree_view: gtk::TreeView,
  cur_lvl_text: String,
  cur_lvl_invalid_text: String,
  cur_lvl_label: gtk::Label,
  cur_lvl_entry: gtk::Entry,
  tgt_lvl_text: String,
  tgt_lvl_invalid_text: String,
  tgt_lvl_label: gtk::Label,
  tgt_lvl_entry: gtk::Entry,
  exp_text: String,
  exp_label: gtk::Label,
  locale: Locale,
}

impl ExperienceGroup {
  fn new(exp_type: ExpType, locale: Locale) -> Rc<ExperienceGroup> {
    let exp_text = match exp_type {
      ExpType::Adventurer => String::from(t!("Adventurer Experience Needed:  {}")),
      ExpType::Producer => String::from(t!("Producer Experience Needed:  {}")),
    };
    let exp_text = exp_text.replace("{}", "<b>{}</b>");

    let cur_lvl_text = String::from(t!("Current Level"));
    let tgt_lvl_text = String::from(t!("Target Level"));
    let group = Rc::new(ExperienceGroup {
      group_box: gtk::Box::new(gtk::Orientation::Vertical, 0),
      tree_view: gtk::TreeView::new(),
      cur_lvl_invalid_text: format!("<span foreground='#FF0000'>{}</span>", cur_lvl_text),
      cur_lvl_label: gtk::Label::new(Some(&cur_lvl_text)),
      cur_lvl_text: cur_lvl_text,
      cur_lvl_entry: gtk::Entry::new(),
      tgt_lvl_invalid_text: format!("<span foreground='#FF0000'>{}</span>", tgt_lvl_text),
      tgt_lvl_label: gtk::Label::new(Some(&tgt_lvl_text)),
      tgt_lvl_text: tgt_lvl_text,
      tgt_lvl_entry: gtk::Entry::new(),
      exp_label: gtk::Label::new(Some(&exp_text.replace("{}", ""))),
      exp_text: exp_text,
      locale: locale,
    });

    // The skill names and their experience multipliers are in CSV resources.
    let skill_csv = match exp_type {
      ExpType::Adventurer => {
        String::from_utf8_lossy(include_bytes!("../res/adventurer_skills.csv"))
      }
      ExpType::Producer => String::from_utf8_lossy(include_bytes!("../res/producer_skills.csv")),
    };

    // Populate a ListStore with the skills and multipliers.
    const LIST_COLS: [glib::Type; 2] = [glib::Type::String, glib::Type::F64];
    let list_store = gtk::ListStore::new(&LIST_COLS);
    for line in skill_csv.lines() {
      let mut parts = line.split(',');
      if let Some(skill) = parts.next() {
        if let Some(mul) = parts.next() {
          if let Ok(mul) = mul.replacen(',', ".", 1).parse::<f64>() {
            let skill = glib::Value::from(skill);
            let mul = glib::Value::from(&mul);
            list_store.insert_with_values(None, &[0, 1], &[&skill, &mul]);
          }
        }
      }
    }

    let cell = gtk::CellRendererText::new();
    let column = gtk::TreeViewColumn::new();
    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", 0);

    // Skills TreeView.
    group.tree_view.set_headers_visible(false);
    group.tree_view.append_column(&column);
    group.tree_view.set_model(Some(&list_store));

    let selection = group.tree_view.get_selection();
    selection.connect_changed(func!(group => move |selection| {
      let active = selection.get_selected().is_some();
      group.cur_lvl_entry.set_sensitive(active);
      group.tgt_lvl_entry.set_sensitive(active);
      group.update();
    }));

    // We need it to scroll.
    let scrolled_window =
      gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scrolled_window.add(&group.tree_view);

    // And we need some separation from the background.
    let skill_frame = gtk::Frame::new(None);
    skill_frame.add(&scrolled_window);

    // Level entry controls.
    group.cur_lvl_entry.set_margins(5, 0, 20, 0);
    group.cur_lvl_entry.set_width_chars(4);
    group.cur_lvl_entry.set_sensitive(false);
    group.tgt_lvl_entry.set_margin_start(5);
    group.tgt_lvl_entry.set_width_chars(4);
    group.tgt_lvl_entry.set_sensitive(false);

    let lvl_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    lvl_box.set_margins(0, 5, 0, 5);
    lvl_box.pack_start(&group.cur_lvl_label, false, true, 0);
    lvl_box.pack_start(&group.cur_lvl_entry, true, true, 0);
    lvl_box.pack_start(&group.tgt_lvl_label, false, true, 0);
    lvl_box.pack_start(&group.tgt_lvl_entry, true, true, 0);

    // Experience needed label.
    group.exp_label.set_use_markup(true);
    group.exp_label.set_halign(gtk::Align::Start);

    // The box to contain it all.
    group.group_box.pack_start(&skill_frame, true, true, 0);
    group.group_box.pack_start(&lvl_box, false, true, 0);
    group.group_box.pack_start(&group.exp_label, false, true, 0);

    group
      .cur_lvl_entry
      .connect_changed(func!(group => move |_| {
        group.update();
      }));

    group
      .tgt_lvl_entry
      .connect_changed(func!(group => move |_| {
        group.update();
      }));

    group
  }

  fn update(&self) {
    let mut cur_valid = true;
    let mut tgt_valid = true;
    let mut result = 0;
    let selection = self.tree_view.get_selection().get_selected();

    if let Some(cur_val_text) = self.cur_lvl_entry.get_text() {
      if let Some(tgt_val_text) = self.tgt_lvl_entry.get_text() {
        let cur_val = if let Ok(val) = cur_val_text.parse::<usize>() {
          val
        } else {
          0
        };

        let tgt_val = if let Ok(val) = tgt_val_text.parse::<usize>() {
          val
        } else {
          0
        };

        if cur_val > 0 || tgt_val > 0 || selection.is_some() {
          cur_valid = cur_val >= 1 && cur_val < 200;
          tgt_valid = tgt_val >= 1 && tgt_val <= 200 && (!cur_valid || tgt_val > cur_val);
          if cur_valid && tgt_valid {
            result = exp_delta(cur_val, tgt_val);
          }
        }
      }
    }

    if cur_valid {
      self.cur_lvl_label.set_text(&self.cur_lvl_text);
    } else {
      self.cur_lvl_label.set_text(&self.cur_lvl_invalid_text);
      self.cur_lvl_label.set_use_markup(true);
    }

    if tgt_valid {
      self.tgt_lvl_label.set_text(&self.tgt_lvl_text);
    } else {
      self.tgt_lvl_label.set_text(&self.tgt_lvl_invalid_text);
      self.tgt_lvl_label.set_use_markup(true);
    }

    if result > 0 {
      if let Some((model, iter)) = selection {
        let mul = model.get_value(&iter, 1);
        if let Ok(Some(mul)) = mul.get::<f64>() {
          result = (mul * result as f64).round() as i64;
          let text = result.to_formatted_string(&self.locale);
          let text = self.exp_text.replace("{}", &text);
          self.exp_label.set_text(&text);
          self.exp_label.set_use_markup(true);
          return;
        }
      }
    }

    self.exp_label.set_text(&self.exp_text.replace("{}", ""));
    self.exp_label.set_use_markup(true);
  }
}
