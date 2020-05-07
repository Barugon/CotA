use crate::util::*;
use gtk::prelude::*;

const PLACES: [&str; 8] = [
  "Blood River",
  "Solace Bridge",
  "Highvale",
  "Brookside",
  "Owl's Head",
  "Westend",
  "Brittany Graveyard",
  "Etceter",
];
const PLACE_LINKS: [&str; 8] = [
  "https://www.shroudoftheavatar.com/map/?map_id=1&amp;poi_id=2758&amp;openPopup=true&amp;z=4",
  "https://www.shroudoftheavatar.com/map/?map_id=1&amp;poi_id=2757&amp;openPopup=true&amp;z=4",
  "https://www.shroudoftheavatar.com/map/?map_id=1&amp;poi_id=999&amp;openPopup=true&amp;z=4",
  "https://www.shroudoftheavatar.com/map/?map_id=1&amp;poi_id=434&amp;openPopup=true&amp;z=4",
  "https://www.shroudoftheavatar.com/map/?map_id=1&amp;poi_id=444&amp;openPopup=true&amp;z=4",
  "https://www.shroudoftheavatar.com/map/?map_id=1&amp;poi_id=587&amp;openPopup=true&amp;z=4",
  "https://www.shroudoftheavatar.com/map/?map_id=1&amp;poi_id=1054&amp;openPopup=true&amp;z=4",
  "https://www.shroudoftheavatar.com/map/?map_id=1&amp;poi_id=632&amp;openPopup=true&amp;z=4",
];
const PHASES: [&str; 8] = [
  "New Moon",
  "Waxing Crescent",
  "First Quarter",
  "Waxing Gibbous",
  "Full Moon",
  "Waning Gibbous",
  "Third Quarter",
  "Waning Crescent",
];

pub struct PortalsPage {
  pub page_box: gtk::Box,
  text_color: Option<gdk::RGBA>,
  rifts_grid: gtk::Grid,
  lost_vale_label: gtk::Label,
}

impl PortalsPage {
  pub fn new() -> PortalsPage {
    let rifts_grid = gtk::Grid::new();
    rifts_grid.set_margins(5, 5, 5, 5);
    for index in 0..8 {
      let place_text = format!(
        "<a href=\"{}\">{}</a>",
        PLACE_LINKS[index as usize],
        t!(PLACES[index as usize])
      );

      let place_label = gtk::Label::new(None);
      place_label.set_text(&place_text);
      place_label.set_use_markup(true);
      place_label.set_track_visited_links(false);
      place_label.set_halign(gtk::Align::Start);
      place_label.set_margins(3, 3, 3, 3);
      rifts_grid.attach(&place_label, 0, index, 1, 1);

      let phase_label = gtk::Label::new(None);
      phase_label.set_halign(gtk::Align::Center);
      phase_label.set_margins(3, 3, 3, 3);
      phase_label.set_hexpand(true);
      rifts_grid.attach(&phase_label, 1, index, 1, 1);

      let rift_label = gtk::Label::new(None);
      rift_label.set_halign(gtk::Align::Start);
      rift_label.set_margins(3, 3, 3, 3);
      rifts_grid.attach(&rift_label, 2, index, 1, 1);
    }

    let lost_vale_label = gtk::Label::new(None);
    let chrono_label = gtk::Label::new(Some(t!("The accuracy of this portal chronometer\ndepends entirely on your system clock.\n\nFor best results, please set your system\nclock to synchronize with Internet time.")));

    let page_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    page_box.pack_start(&rifts_grid, false, true, 0);
    page_box.pack_start(&lost_vale_label, false, true, 10);
    page_box.pack_start(&chrono_label, true, true, 0);

    PortalsPage {
      page_box: page_box,
      text_color: lost_vale_label.get_text_color(),
      rifts_grid: rifts_grid,
      lost_vale_label: lost_vale_label,
    }
  }

  pub fn update(&self) {
    self.update_lunar_rifts();
    self.update_lost_vale();
  }

  fn update_lunar_rifts(&self) {
    // Get the lunar phase.
    let phase = get_lunar_phase();
    let mut rift = phase as i32;

    // Get the time remaining for the active lunar rift.
    let remain = 8.75 * (1.0 - (phase - rift as f64));
    let mut minutes = remain as i32;
    let mut seconds = (60.0 * (remain - minutes as f64) + 0.5) as i32;
    if seconds > 59 {
      minutes += 1;
      seconds -= 60;
    }

    let opens_text = t!("Opens in {}m {}s").replace("{}", "<tt>{}</tt>");
    let closes_text = t!("Closes in {}m {}s").replace("{}", "<tt>{}</tt>");
    let phase_markup;
    let rift_markup;
    if let Some(color) = self.text_color {
      let color = html_color(&color, 0.4);
      phase_markup = format!("<span foreground='{}'>{{}}</span>", color);
      rift_markup = format!("<span foreground='{}'>{}</span>", color, opens_text);
    } else {
      phase_markup = String::from("{}");
      rift_markup = opens_text;
    }

    // The first rift is the active one.
    if let Some(widget) = self.rifts_grid.get_child_at(1, rift) {
      if let Ok(phase_label) = widget.dynamic_cast::<gtk::Label>() {
        phase_label.set_text(t!(PHASES[rift as usize]));
      }
    }

    if let Some(widget) = self.rifts_grid.get_child_at(2, rift) {
      if let Ok(rifts_label) = widget.dynamic_cast::<gtk::Label>() {
        let minutes = format!("{:02}", minutes);
        let seconds = format!("{:02}", seconds);
        let closes_text = closes_text
          .replacen("{}", &minutes, 1)
          .replacen("{}", &seconds, 1);
        rifts_label.set_text(&closes_text);
        rifts_label.set_use_markup(true);
      }
    }

    for _ in 1..8 {
      rift += 1;
      if rift > 7 {
        rift = 0;
      }

      // Draw the inactive lunar rifts in a less pronounced way.
      if let Some(widget) = self.rifts_grid.get_child_at(1, rift) {
        if let Ok(phase_label) = widget.dynamic_cast::<gtk::Label>() {
          phase_label.set_text(&phase_markup.replacen("{}", t!(PHASES[rift as usize]), 1));
          phase_label.set_use_markup(true);
        }
      }

      if let Some(widget) = self.rifts_grid.get_child_at(2, rift) {
        if let Ok(rift_label) = widget.dynamic_cast::<gtk::Label>() {
          let minutes = format!("{:02}", minutes);
          let seconds = format!("{:02}", seconds);
          let opens_text = rift_markup
            .replacen("{}", &minutes, 1)
            .replacen("{}", &seconds, 1);
          rift_label.set_text(&opens_text);
          rift_label.set_use_markup(true);
        }
      }

      // Add time for the next lunar rift.
      minutes += 8;
      seconds += 45;
      if seconds > 59 {
        minutes += 1;
        seconds -= 60;
      }
    }
  }

  fn update_lost_vale(&self) {
    let countdown = get_lost_vale_countdown();
    if countdown < 0.0 {
      let remaining = countdown.abs();
      let minutes = remaining as i32;
      let seconds = (60.0 * (remaining - minutes as f64) + 0.5) as i32;
      let minutes = format!("<tt>{:02}</tt>", minutes);
      let seconds = format!("<tt>{:02}</tt>", seconds);
      let closes_text = t!("Lost Vale closes in {}m {}s")
        .replacen("{}", &minutes, 1)
        .replacen("{}", &seconds, 1);

      // The Lost Vale is currently open.
      self.lost_vale_label.set_text(&closes_text);
    } else {
      let minutes = countdown as i32;
      let seconds = (60.0 * (countdown - minutes as f64) + 0.5) as i32;
      let hours = format!("<tt>{:02}</tt>", minutes / 60);
      let minutes = format!("<tt>{:02}</tt>", minutes % 60);
      let seconds = format!("<tt>{:02}</tt>", seconds);
      let opens_text = t!("Lost Vale opens in {}h {}m {}s")
        .replacen("{}", &hours, 1)
        .replacen("{}", &minutes, 1)
        .replacen("{}", &seconds, 1);
      if let Some(color) = self.text_color {
        self.lost_vale_label.set_text(&format!(
          "<span foreground='{}'>{}</span>",
          &html_color(&color, 0.4),
          &opens_text
        ));
      } else {
        self.lost_vale_label.set_text(&opens_text);
      }
    }

    self.lost_vale_label.set_use_markup(true);
  }
}
