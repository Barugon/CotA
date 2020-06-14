use crate::util::*;
use gdnative::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Portals {
  places: [NodePath; 8],
  phases: [NodePath; 8],
  times: [NodePath; 8],
  lost_vale_name: NodePath,
  lost_vale_time: NodePath,
  color_name: GodotString,
  opened_color: Variant,
  closed_color: Variant,
  opened_rift_color: Variant,
  closed_rift_color: Variant,
  opened_vale_color: Variant,
  closed_vale_color: Variant,
  timer: NodePath,
}

#[methods]
impl Portals {
  fn _init(_owner: Node) -> Self {
    Portals {
      places: [
        NodePath::from("VBox/Grid/BloodRiverName"),
        NodePath::from("VBox/Grid/SolaceBridgeName"),
        NodePath::from("VBox/Grid/HighvaleName"),
        NodePath::from("VBox/Grid/BrooksideName"),
        NodePath::from("VBox/Grid/OwlsHeadName"),
        NodePath::from("VBox/Grid/WestendName"),
        NodePath::from("VBox/Grid/BrittanyGraveyardName"),
        NodePath::from("VBox/Grid/EtceterName"),
      ],
      phases: [
        NodePath::from("VBox/Grid/BloodRiverPhase"),
        NodePath::from("VBox/Grid/SolaceBridgePhase"),
        NodePath::from("VBox/Grid/HighvalePhase"),
        NodePath::from("VBox/Grid/BrooksidePhase"),
        NodePath::from("VBox/Grid/OwlsHeadPhase"),
        NodePath::from("VBox/Grid/WestendPhase"),
        NodePath::from("VBox/Grid/BrittanyGraveyardPhase"),
        NodePath::from("VBox/Grid/EtceterPhase"),
      ],
      times: [
        NodePath::from("VBox/Grid/BloodRiverTime"),
        NodePath::from("VBox/Grid/SolaceBridgeTime"),
        NodePath::from("VBox/Grid/HighvaleTime"),
        NodePath::from("VBox/Grid/BrooksideTime"),
        NodePath::from("VBox/Grid/OwlsHeadTime"),
        NodePath::from("VBox/Grid/WestendTime"),
        NodePath::from("VBox/Grid/BrittanyGraveyardTime"),
        NodePath::from("VBox/Grid/EtceterTime"),
      ],
      lost_vale_name: NodePath::from("VBox/HBox/LostValeName"),
      lost_vale_time: NodePath::from("VBox/HBox/LostValeTime"),
      color_name: GodotString::from("custom_colors/font_color"),
      opened_color: Variant::from_color(&Color::rgb(1.0, 1.0, 1.0)),
      closed_color: Variant::from_color(&Color::rgb(0.5, 0.5, 0.5)),
      opened_rift_color: Variant::from_color(&Color::rgb(0.7, 0.9, 1.0)),
      closed_rift_color: Variant::from_color(&Color::rgb(0.4, 0.6, 0.7)),
      opened_vale_color: Variant::from_color(&Color::rgb(0.9, 1.0, 0.7)),
      closed_vale_color: Variant::from_color(&Color::rgb(0.6, 0.7, 0.4)),
      timer: NodePath::from("Timer"),
    }
  }

  #[export]
  fn _ready(&self, owner: Node) {
    // Connect the timer.
    owner.connect_to(&self.timer, "timeout", "update");
  }

  #[export]
  fn update(&self, owner: Node) {
    unsafe {
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
      let mut place_label = some!(owner.get_node_as::<Label>(&self.places[rift]));
      place_label.set(self.color_name.new_ref(), self.opened_rift_color.clone());

      let mut phase_label = some!(owner.get_node_as::<Label>(&self.phases[rift]));
      phase_label.set(self.color_name.new_ref(), self.opened_color.clone());

      let mut time_label = some!(owner.get_node_as::<Label>(&self.times[rift]));
      time_label.set(self.color_name.new_ref(), self.opened_color.clone());
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
        let mut place_label = some!(owner.get_node_as::<Label>(&self.places[rift]));
        place_label.set(self.color_name.new_ref(), self.closed_rift_color.clone());

        let mut phase_label = some!(owner.get_node_as::<Label>(&self.phases[rift]));
        phase_label.set(self.color_name.new_ref(), self.closed_color.clone());

        let mut time_label = some!(owner.get_node_as::<Label>(&self.times[rift]));
        time_label.set(self.color_name.new_ref(), self.closed_color.clone());
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
      let mut lost_vale_name = some!(owner.get_node_as::<Label>(&self.lost_vale_name));
      let mut lost_vale_time = some!(owner.get_node_as::<Label>(&self.lost_vale_time));
      let countdown = get_lost_vale_countdown();
      if countdown < 0.0 {
        let remaining = countdown.abs();
        let minutes = remaining as i32;
        let seconds = (60.0 * (remaining - minutes as f64) + 0.5) as i32;

        // The Lost Vale is currently open.
        lost_vale_name.set(self.color_name.new_ref(), self.opened_vale_color.clone());
        lost_vale_time.set(self.color_name.new_ref(), self.opened_color.clone());
        lost_vale_time.set_text(GodotString::from(format!(
          "closes in {:02}m {:02}s",
          minutes, seconds
        )));
      } else {
        let minutes = countdown as i32;
        let seconds = (60.0 * (countdown - minutes as f64) + 0.5) as i32;

        // The Lost Vale is currently closed.
        lost_vale_name.set(self.color_name.new_ref(), self.closed_vale_color.clone());
        lost_vale_time.set(self.color_name.new_ref(), self.closed_color.clone());
        lost_vale_time.set_text(GodotString::from(format!(
          "opens in {:02}h {:02}m {:02}s",
          minutes / 60,
          minutes % 60,
          seconds
        )));
      }
    }
  }
}
