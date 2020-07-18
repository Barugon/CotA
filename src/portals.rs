use crate::util::*;
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
