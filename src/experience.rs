use crate::constants::*;
use crate::util::*;
use gdnative::api::*;
use gdnative::prelude::*;
use num_format::{Locale, ToFormattedString};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Experience {
  tree: GodotString,
  current: GodotString,
  target: GodotString,
  result: GodotString,
  color_name: GodotString,
  good_color: Variant,
  bad_color: Variant,
  locale: Locale,
  selected: AtomicBool,
}

#[methods]
impl Experience {
  fn new(_owner: &Node) -> Self {
    Experience {
      tree: GodotString::from("VBox/Panel/Tree"),
      current: GodotString::from("VBox/LvlHBox/CurrentEdit"),
      target: GodotString::from("VBox/LvlHBox/TargetEdit"),
      result: GodotString::from("VBox/ResHBox/Result"),
      color_name: GodotString::from("custom_colors/font_color"),
      good_color: Variant::from_color(&Color::rgb(0.81, 0.81, 0.81)),
      bad_color: Variant::from_color(&Color::rgb(1.0, 0.0, 0.0)),
      locale: get_locale(),
      selected: AtomicBool::new(false),
    }
  }

  #[export]
  fn _ready(&self, owner: TRef<Node>) {
    let path = owner.get_path().to_string();
    if path.ends_with("/AdvPanel") {
      self.populate_tree(owner, ADVENTURER_SKILLS)
    } else if path.ends_with("/ProPanel") {
      self.populate_tree(owner, PRODUCER_SKILLS)
    }

    // Connect tree item_selected.
    owner.connect_to(&self.tree, "item_selected", "item_selected");

    // Connect tree item_collapsed.
    owner.connect_to(&self.tree, "item_collapsed", "item_collapsed");

    // Connect current text_changed.
    owner.connect_to(&self.current, "text_changed", "text_changed");

    // Connect target text_changed.
    owner.connect_to(&self.target, "text_changed", "text_changed");

    if let Some(tree) = owner.get_node_as::<Tree>(&self.tree) {
      tree.set_column_expand(0, true);
      tree.set_column_min_width(0, 3);
      // tree.set_column_title(0, GodotString::from("Skill"));
      // tree.set_column_title(1, GodotString::from("Mul"));
      // tree.set_column_title(2, GodotString::from("ID"));
      // tree.set_column_titles_visible(true);
    }
  }

  #[export]
  fn item_selected(&self, owner: TRef<Node>) {
    let tree = some!(owner.get_node_as::<Tree>(&self.tree));
    self.selected.store(true, Ordering::Relaxed);
    self.update(owner, tree);
  }

  #[export]
  fn item_collapsed(&self, owner: TRef<Node>, _item: Ref<TreeItem>) {
    // This is a bit kludgy but the item is actually still selected
    // (but not highlighted) after the parent is collapsed.
    let tree = some!(owner.get_node_as::<Tree>(&self.tree));
    if let Some(item) = tree.get_selected() {
      let item = item.to_ref();
      if let Some(parent) = item.get_parent() {
        let parent = parent.to_ref();
        if parent.is_collapsed() && self.selected.swap(false, Ordering::Relaxed) {
          self.update(owner, tree);
        }
      }
    }
  }

  #[export]
  fn text_changed(&self, owner: TRef<Node>, _text: GodotString) {
    let tree = some!(owner.get_node_as::<Tree>(&self.tree));
    self.update(owner, tree);
  }

  fn update(&self, owner: TRef<Node>, tree: TRef<Tree>) {
    let mut text = GodotString::new();
    let result = some!(owner.get_node_as::<Label>(&self.result));

    if self.selected.load(Ordering::Relaxed) {
      let current = some!(owner.get_node_as::<LineEdit>(&self.current));
      let target = some!(owner.get_node_as::<LineEdit>(&self.target));

      if let Some(item) = tree.get_selected() {
        let item = item.to_ref();
        let cur = current
          .text()
          .to_utf8()
          .as_str()
          .parse::<usize>()
          .unwrap_or(0);
        let cur_valid = cur >= 1 && cur < 200;
        if cur_valid {
          current.set(self.color_name.clone(), self.good_color.clone());
        } else {
          current.set(self.color_name.clone(), self.bad_color.clone());
        }

        let tgt = target
          .text()
          .to_utf8()
          .as_str()
          .parse::<usize>()
          .unwrap_or(0);
        let tgt_valid = tgt >= 1 && tgt <= 200 && (!cur_valid || tgt > cur);
        if tgt_valid {
          target.set(self.color_name.clone(), self.good_color.clone());
        } else {
          target.set(self.color_name.clone(), self.bad_color.clone());
        }

        if let Ok(mul) = item
          .get_text(1)
          .to_utf8()
          .as_str()
          .trim_end_matches('x')
          .parse::<f64>()
        {
          if cur_valid && tgt_valid {
            let val = SKILL_EXP_VALUES[tgt - 1] - SKILL_EXP_VALUES[cur - 1];
            let val = (val as f64 * mul).ceil() as i64;
            text = GodotString::from(val.to_formatted_string(&self.locale));
          }
        }
      }
    }

    result.set_text(text);
  }

  fn populate_tree(&self, owner: TRef<Node>, csv: &str) {
    let tree = some!(owner.get_node_as::<Tree>(&self.tree));
    let skill_color = Color::rgb(0.4, 0.6, 0.7);
    let info_color = Color::rgb(0.5, 0.5, 0.5);
    let root = some!(tree.create_item(Object::null(), -1));
    let mut parent = None;
    let mut category = "";

    for line in csv.lines() {
      let mut iter = line.split(',');

      // Category.
      if let Some(text) = iter.next() {
        if category != text {
          category = text;
          if let Some(item) = tree.create_item(root, -1) {
            let item = item.to_ref();
            item.set_selectable(0, false);
            item.set_selectable(1, false);
            item.set_selectable(2, false);
            item.set_text(0, GodotString::from(text));
            item.set_collapsed(true);
            parent = Some(item);
          }
        }

        if let Some(parent) = parent {
          if let Some(item) = tree.create_item(parent, -1) {
            let item = item.to_ref();
            // Skill name.
            if let Some(text) = iter.next() {
              item.set_custom_color(0, skill_color);
              item.set_text(0, GodotString::from(text));
            }
            // Experience multiplier.
            if let Some(text) = iter.next() {
              item.set_custom_color(1, info_color);
              item.set_text(1, GodotString::from(format!("{}x", text)));
            }
            // Skill ID.
            if let Some(text) = iter.next() {
              item.set_custom_color(2, info_color);
              item.set_text(2, GodotString::from(text));
            }
          }
        }
      }
    }
  }
}
