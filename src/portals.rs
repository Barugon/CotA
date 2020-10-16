use crate::util::*;
use chrono::{TimeZone, Utc};
use gdnative::api::*;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Portals {
  places: [GodotString; 8],
  phases: [GodotString; 8],
  times: [GodotString; 8],
  lost_vale_name: GodotString,
  lost_vale_time: GodotString,
  color_name: GodotString,
  opened_color: Variant,
  closed_color: Variant,
  opened_rift_color: Variant,
  closed_rift_color: Variant,
  opened_vale_color: Variant,
  closed_vale_color: Variant,
  timer: GodotString,
}

#[methods]
impl Portals {
  fn new(_owner: &Node) -> Self {
    Portals {
      places: [
        GodotString::from("VBox/Grid/BloodRiverName"),
        GodotString::from("VBox/Grid/SolaceBridgeName"),
        GodotString::from("VBox/Grid/HighvaleName"),
        GodotString::from("VBox/Grid/BrooksideName"),
        GodotString::from("VBox/Grid/OwlsHeadName"),
        GodotString::from("VBox/Grid/WestendName"),
        GodotString::from("VBox/Grid/BrittanyGraveyardName"),
        GodotString::from("VBox/Grid/EtceterName"),
      ],
      phases: [
        GodotString::from("VBox/Grid/BloodRiverPhase"),
        GodotString::from("VBox/Grid/SolaceBridgePhase"),
        GodotString::from("VBox/Grid/HighvalePhase"),
        GodotString::from("VBox/Grid/BrooksidePhase"),
        GodotString::from("VBox/Grid/OwlsHeadPhase"),
        GodotString::from("VBox/Grid/WestendPhase"),
        GodotString::from("VBox/Grid/BrittanyGraveyardPhase"),
        GodotString::from("VBox/Grid/EtceterPhase"),
      ],
      times: [
        GodotString::from("VBox/Grid/BloodRiverTime"),
        GodotString::from("VBox/Grid/SolaceBridgeTime"),
        GodotString::from("VBox/Grid/HighvaleTime"),
        GodotString::from("VBox/Grid/BrooksideTime"),
        GodotString::from("VBox/Grid/OwlsHeadTime"),
        GodotString::from("VBox/Grid/WestendTime"),
        GodotString::from("VBox/Grid/BrittanyGraveyardTime"),
        GodotString::from("VBox/Grid/EtceterTime"),
      ],
      lost_vale_name: GodotString::from("VBox/HBox/LostValeName"),
      lost_vale_time: GodotString::from("VBox/HBox/LostValeTime"),
      color_name: GodotString::from("custom_colors/font_color"),
      opened_color: Variant::from_color(&Color::rgb(1.0, 1.0, 1.0)),
      closed_color: Variant::from_color(&Color::rgb(0.5, 0.5, 0.5)),
      opened_rift_color: Variant::from_color(&Color::rgb(0.7, 0.9, 1.0)),
      closed_rift_color: Variant::from_color(&Color::rgb(0.4, 0.6, 0.7)),
      opened_vale_color: Variant::from_color(&Color::rgb(0.9, 1.0, 0.7)),
      closed_vale_color: Variant::from_color(&Color::rgb(0.6, 0.7, 0.4)),
      timer: GodotString::from("Timer"),
    }
  }

  #[export]
  fn _ready(&self, owner: TRef<Node>) {
    // Connect the timer.
    owner.connect_to(&self.timer, "timeout", "update");
  }

  #[export]
  fn update(&self, owner: TRef<Node>) {
    // Get the lunar phase.
    let phase = get_lunar_phase();
    let mut rift = phase as usize;

    // Get the time remaining for the active lunar rift.
    let remain = 8.75 * (1.0 - (phase - rift as f64));
    let mut minutes = remain as i32;
    let mut seconds = (60.0 * (remain - minutes as f64) + 0.5) as i32;
    if seconds > 59 {
      minutes += 1;
      seconds -= 60;
    }

    // The first rift is the active one.
    let place_label = some!(owner.get_node_as::<Label>(&self.places[rift]));
    place_label.set(self.color_name.clone(), self.opened_rift_color.clone());

    let phase_label = some!(owner.get_node_as::<Label>(&self.phases[rift]));
    phase_label.set(self.color_name.clone(), self.opened_color.clone());

    let time_label = some!(owner.get_node_as::<Label>(&self.times[rift]));
    time_label.set(self.color_name.clone(), self.opened_color.clone());
    time_label.set_text(GodotString::from(format!(
      "closes in {:02}m {:02}s",
      minutes, seconds
    )));

    for _ in 1..8 {
      rift += 1;
      if rift > 7 {
        rift = 0;
      }

      // Draw the inactive lunar rifts in a less pronounced way.
      let place_label = some!(owner.get_node_as::<Label>(&self.places[rift]));
      place_label.set(self.color_name.clone(), self.closed_rift_color.clone());

      let phase_label = some!(owner.get_node_as::<Label>(&self.phases[rift]));
      phase_label.set(self.color_name.clone(), self.closed_color.clone());

      let time_label = some!(owner.get_node_as::<Label>(&self.times[rift]));
      time_label.set(self.color_name.clone(), self.closed_color.clone());
      time_label.set_text(GodotString::from(format!(
        "opens in {:02}m {:02}s",
        minutes, seconds
      )));

      // Add time for the next lunar rift.
      minutes += 8;
      seconds += 45;
      if seconds > 59 {
        minutes += 1;
        seconds -= 60;
      }
    }

    // Update the Lost Vale countdown.
    let lost_vale_name = some!(owner.get_node_as::<Label>(&self.lost_vale_name));
    let lost_vale_time = some!(owner.get_node_as::<Label>(&self.lost_vale_time));
    let countdown = get_lost_vale_countdown();
    if countdown < 0.0 {
      let remaining = countdown.abs();
      let minutes = remaining as i32;
      let seconds = (60.0 * (remaining - minutes as f64) + 0.5) as i32;

      // The Lost Vale is currently open.
      lost_vale_name.set(self.color_name.clone(), self.opened_vale_color.clone());
      lost_vale_time.set(self.color_name.clone(), self.opened_color.clone());
      lost_vale_time.set_text(GodotString::from(format!(
        "closes in {:02}m {:02}s",
        minutes, seconds
      )));
    } else {
      let minutes = countdown as i32;
      let seconds = (60.0 * (countdown - minutes as f64) + 0.5) as i32;

      // The Lost Vale is currently closed.
      lost_vale_name.set(self.color_name.clone(), self.closed_vale_color.clone());
      lost_vale_time.set(self.color_name.clone(), self.closed_color.clone());
      lost_vale_time.set_text(GodotString::from(format!(
        "opens in {:02}h {:02}m {:02}s",
        minutes / 60,
        minutes % 60,
        seconds
      )));
    }
  }
}

/// Get the current lunar phase as f64.
fn get_lunar_phase() -> f64 {
  // Get the elapsed time since the lunar rift epoch.
  let dur = Utc::now() - Utc.ymd(1997, 9, 2).and_hms(0, 0, 0);

  // Calculate the lunar phase from the duration. Each phase is 525 seconds and there are 8 phases, for a total of 4200
  // seconds per lunar cycle.
  (dur.num_seconds() % 4200) as f64 / 525.0
}

/// Get the current Lost Vale countdown (in minutes) as f64.
fn get_lost_vale_countdown() -> f64 {
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
