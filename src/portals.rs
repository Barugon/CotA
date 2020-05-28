use crate::util::*;
use gdnative::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Portals {
  places: [NodePath; 8],
  phases: [NodePath; 8],
  times: [NodePath; 8],
  lost_vale: NodePath,
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
      lost_vale: NodePath::from_str("VBox/HBox/LostVale"),
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

    unsafe {
      // The first rift is the active one.
      let mut place_label = some!(owner.get_node_as::<RichTextLabel>(&self.places[rift]));
      place_label.set_bbcode(GodotString::from_str(&format!(
        "[color=#B8E3FF]{}[/color]",
        PLACES[rift]
      )));

      let mut phase_label = some!(owner.get_node_as::<RichTextLabel>(&self.phases[rift]));
      phase_label.set_bbcode(GodotString::from_str(&format!(
        "[center][color=#FFFFFF]{}[/color][/center]",
        PHASES[rift]
      )));

      let mut time_label = some!(owner.get_node_as::<RichTextLabel>(&self.times[rift]));
      time_label.set_bbcode(GodotString::from_str(&format!(
        "[right][color=#FFFFFF]closes in {:02}m {:02}s[/color][/right]",
        minutes, seconds
      )));

      for _ in 1..8 {
        rift += 1;
        if rift > 7 {
          rift = 0;
        }

        // Draw the inactive lunar rifts in a less pronounced way.
        let mut place_label = some!(owner.get_node_as::<RichTextLabel>(&self.places[rift]));
        place_label.set_bbcode(GodotString::from_str(&format!(
          "[color=#3C6880]{}[/color]",
          PLACES[rift]
        )));

        let mut phase_label = some!(owner.get_node_as::<RichTextLabel>(&self.phases[rift]));
        phase_label.set_bbcode(GodotString::from_str(&format!(
          "[center][color=#808080]{}[/color][/center]",
          PHASES[rift]
        )));

        let mut time_label = some!(owner.get_node_as::<RichTextLabel>(&self.times[rift]));
        time_label.set_bbcode(GodotString::from_str(&format!(
          "[right][color=#808080]opens in {:02}m {:02}s[/color][/right]",
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
      let mut lost_vale_label = some!(owner.get_node_as::<RichTextLabel>(&self.lost_vale));
      let countdown = get_lost_vale_countdown();
      if countdown < 0.0 {
        let remaining = countdown.abs();
        let minutes = remaining as i32;
        let seconds = (60.0 * (remaining - minutes as f64) + 0.5) as i32;

        // The Lost Vale is currently open.
        lost_vale_label.set_bbcode(GodotString::from_str(&format!(
          "[center][color=#B8FFE3]Lost Vale[/color] [color=#FFFFFF]closes in {:02}m {:02}s[/color][/center]",
          minutes, seconds
        )));
      } else {
        let minutes = countdown as i32;
        let seconds = (60.0 * (countdown - minutes as f64) + 0.5) as i32;

        // The Lost Vale is currently closed.
        lost_vale_label.set_bbcode(GodotString::from_str(&format!(
          "[center][color=#3C8068]Lost Vale[/color] [color=#808080]opens in {:02}h {:02}m {:02}s[/color][/center]",
          minutes / 60, minutes % 60, seconds
        )));
      }
    }
  }
}
