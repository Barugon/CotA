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
        NodePath::from_str("VBox/Grid/BloodRiverName"),
        NodePath::from_str("VBox/Grid/SolaceBridgeName"),
        NodePath::from_str("VBox/Grid/HighvaleName"),
        NodePath::from_str("VBox/Grid/BrooksideName"),
        NodePath::from_str("VBox/Grid/OwlsHeadName"),
        NodePath::from_str("VBox/Grid/WestendName"),
        NodePath::from_str("VBox/Grid/BrittanyGraveyardName"),
        NodePath::from_str("VBox/Grid/EtceterName"),
      ],
      phases: [
        NodePath::from_str("VBox/Grid/BloodRiverPhase"),
        NodePath::from_str("VBox/Grid/SolaceBridgePhase"),
        NodePath::from_str("VBox/Grid/HighvalePhase"),
        NodePath::from_str("VBox/Grid/BrooksidePhase"),
        NodePath::from_str("VBox/Grid/OwlsHeadPhase"),
        NodePath::from_str("VBox/Grid/WestendPhase"),
        NodePath::from_str("VBox/Grid/BrittanyGraveyardPhase"),
        NodePath::from_str("VBox/Grid/EtceterPhase"),
      ],
      times: [
        NodePath::from_str("VBox/Grid/BloodRiverTime"),
        NodePath::from_str("VBox/Grid/SolaceBridgeTime"),
        NodePath::from_str("VBox/Grid/HighvaleTime"),
        NodePath::from_str("VBox/Grid/BrooksideTime"),
        NodePath::from_str("VBox/Grid/OwlsHeadTime"),
        NodePath::from_str("VBox/Grid/WestendTime"),
        NodePath::from_str("VBox/Grid/BrittanyGraveyardTime"),
        NodePath::from_str("VBox/Grid/EtceterTime"),
      ],
      lost_vale_name: NodePath::from_str("VBox/HBox/LostValeName"),
      lost_vale_time: NodePath::from_str("VBox/HBox/LostValeTime"),
      color_name: GodotString::from_str("custom_colors/font_color"),
      opened_color: Variant::from_color(&Color::rgb(1.0, 1.0, 1.0)),
      closed_color: Variant::from_color(&Color::rgb(0.5, 0.5, 0.5)),
      opened_rift_color: Variant::from_color(&Color::rgb(0.7, 0.9, 1.0)),
      closed_rift_color: Variant::from_color(&Color::rgb(0.4, 0.6, 0.7)),
      opened_vale_color: Variant::from_color(&Color::rgb(0.9, 1.0, 0.7)),
      closed_vale_color: Variant::from_color(&Color::rgb(0.6, 0.7, 0.4)),
      timer: NodePath::from_str("Timer"),
    }
  }

  #[export]
  fn _ready(&self, owner: Node) {
    // Connect the timer.
    owner.connect_to(&self.timer, "timeout", "update");
  }

  #[export]
  fn update(&self, owner: Node) {
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

    // #B8FFE3
    unsafe {
      // The first rift is the active one.
      let mut place_label = some!(owner.get_node_as::<Label>(&self.places[rift]));
      place_label.set(self.color_name.new_ref(), self.opened_rift_color.clone());

      let mut phase_label = some!(owner.get_node_as::<Label>(&self.phases[rift]));
      phase_label.set(self.color_name.new_ref(), self.opened_color.clone());

      let mut time_label = some!(owner.get_node_as::<Label>(&self.times[rift]));
      time_label.set(self.color_name.new_ref(), self.opened_color.clone());
      time_label.set_text(GodotString::from_str(&format!(
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
        time_label.set_text(GodotString::from_str(&format!(
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
        lost_vale_time.set_text(GodotString::from_str(&format!(
          "closes in {:02}m {:02}s",
          minutes, seconds
        )));
      } else {
        let minutes = countdown as i32;
        let seconds = (60.0 * (countdown - minutes as f64) + 0.5) as i32;

        // The Lost Vale is currently closed.
        lost_vale_name.set(self.color_name.new_ref(), self.closed_vale_color.clone());
        lost_vale_time.set(self.color_name.new_ref(), self.closed_color.clone());
        lost_vale_time.set_text(GodotString::from_str(&format!(
          "opens in {:02}h {:02}m {:02}s",
          minutes / 60,
          minutes % 60,
          seconds
        )));
      }
    }
  }
}
