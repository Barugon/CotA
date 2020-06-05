use crate::util::*;
use gdnative::*;
use num_format::{Locale, ToFormattedString};

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Experience {
  tree: NodePath,
  current: NodePath,
  target: NodePath,
  result: NodePath,
  color_name: GodotString,
  good_color: Variant,
  bad_color: Variant,
  locale: Locale,
}

#[methods]
impl Experience {
  fn _init(_owner: Node) -> Self {
    Experience {
      tree: NodePath::from_str("VBox/Panel/Tree"),
      current: NodePath::from_str("VBox/LvlHBox/CurrentEdit"),
      target: NodePath::from_str("VBox/LvlHBox/TargetEdit"),
      result: NodePath::from_str("VBox/ResHBox/Result"),
      color_name: GodotString::from_str("custom_colors/font_color"),
      good_color: Variant::from_color(&Color::rgb(0.81, 0.81, 0.81)),
      bad_color: Variant::from_color(&Color::rgb(1.0, 0.0, 0.0)),
      locale: get_locale(),
    }
  }

  #[export]
  fn _ready(&self, owner: Node) {
    unsafe {
      let path = owner.get_path().to_string();
      if path.ends_with("/AdvPanel") {
        self.populate_tree(owner, include_str!("res/adventurer_skills.csv"))
      } else if path.ends_with("/ProPanel") {
        self.populate_tree(owner, include_str!("res/producer_skills.csv"))
      }

      // Connect tree item_selected.
      owner.connect_to(&self.tree, "item_selected", "update");

      // Connect current text_changed.
      owner.connect_to(&self.current, "text_changed", "text_changed");

      // Connect target text_changed.
      owner.connect_to(&self.target, "text_changed", "text_changed");
    }
  }

  #[export]
  fn text_changed(&self, owner: Node, _text: GodotString) {
    self.update(owner);
  }

  #[export]
  fn update(&self, owner: Node) {
    let tree = some!(owner.get_node_as::<Tree>(&self.tree));
    let mut current = some!(owner.get_node_as::<LineEdit>(&self.current));
    let mut target = some!(owner.get_node_as::<LineEdit>(&self.target));
    let mut result = some!(owner.get_node_as::<Label>(&self.result));
    let mut text = GodotString::new();

    unsafe {
      if let Some(item) = tree.get_selected() {
        current.set_editable(true);
        current.set_focus_mode(Control::FOCUS_ALL);
        target.set_editable(true);
        target.set_focus_mode(Control::FOCUS_ALL);

        let cur = current
          .get_text()
          .to_utf8()
          .as_str()
          .parse::<usize>()
          .unwrap_or(0);

        let tgt = target
          .get_text()
          .to_utf8()
          .as_str()
          .parse::<usize>()
          .unwrap_or(0);

        let cur_valid = cur >= 1 && cur < 200;
        if cur_valid {
          current.set(self.color_name.new_ref(), self.good_color.clone());
        } else {
          current.set(self.color_name.new_ref(), self.bad_color.clone());
        }

        let tgt_valid = tgt >= 1 && tgt <= 200 && (!cur_valid || tgt > cur);
        if tgt_valid {
          target.set(self.color_name.new_ref(), self.good_color.clone());
        } else {
          target.set(self.color_name.new_ref(), self.bad_color.clone());
        }

        if let Ok(mul) = item
          .get_text(1)
          .to_utf8()
          .as_str()
          .trim_end_matches('x')
          .parse::<f64>()
        {
          if cur_valid && tgt_valid {
            const EXP_VALUES: [i64; 200] = include!("res/exp_values");
            let val = (mul * (EXP_VALUES[tgt - 1] - EXP_VALUES[cur - 1]) as f64).round() as i64;
            text = GodotString::from_str(&val.to_formatted_string(&self.locale));
          }
        }
      } else {
        current.set_focus_mode(Control::FOCUS_NONE);
        current.set_editable(false);
        target.set_focus_mode(Control::FOCUS_NONE);
        target.set_editable(false);
      }

      result.set_text(text);
    }
  }

  fn populate_tree(&self, owner: Node, csv: &str) {
    let mut tree = some!(owner.get_node_as::<Tree>(&self.tree));
    let skill_color = Color::rgb(0.4, 0.6, 0.7);
    let info_color = Color::rgb(0.5, 0.5, 0.5);

    unsafe {
      tree.set_column_expand(0, true);
      tree.set_column_min_width(0, 3);

      if let Some(parent) = tree.create_item(None, -1) {
        for line in csv.lines() {
          if let Some(mut item) = tree.create_item(parent.cast::<Object>(), -1) {
            let mut iter = line.split(',');

            // Skill name.
            if let Some(text) = iter.next() {
              item.set_custom_color(0, skill_color);
              item.set_text(0, GodotString::from_str(text));
            }

            // Experience multiplier.
            if let Some(text) = iter.next() {
              item.set_custom_color(1, info_color);
              item.set_text(1, GodotString::from_str(&format!("{}x", text)));
            }

            // Skill ID.
            if let Some(text) = iter.next() {
              item.set_custom_color(2, info_color);
              item.set_text(2, GodotString::from_str(text));
            }
          }
        }
      }
    }
  }
}
