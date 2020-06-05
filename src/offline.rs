use crate::util::*;
use gdnative::*;

enum SkillTree {
  Adventurer(NodePath),
  Producer(NodePath),
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Offline {
  save: NodePath,
  gold: NodePath,
  adventurer: SkillTree,
  producer: SkillTree,
}

#[methods]
impl Offline {
  fn _init(_owner: Node) -> Self {
    Offline {
      save: NodePath::from_str("HBox/SaveButton"),
      gold: NodePath::from_str("HBox/SpinBox"),
      adventurer: SkillTree::Adventurer(NodePath::from_str("AdvPanel/Tree")),
      producer: SkillTree::Producer(NodePath::from_str("ProPanel/Tree")),
    }
  }

  #[export]
  fn _ready(&self, owner: Node) {
    self.connect_item_changed(owner, &self.adventurer);
    self.connect_item_changed(owner, &self.producer);

    //
    if let Some(mut gold) = owner.get_node_as::<SpinBox>(&self.gold) {
      unsafe {
        gold.set_editable(true);
        gold.set_focus_mode(Control::FOCUS_ALL);
      }
    }
    self.populate_tree(owner, &self.adventurer);
    self.populate_tree(owner, &self.producer);
  }

  #[export]
  fn item_changed(&self, owner: Node) {
    if let Some(mut button) = owner.get_node_as::<Button>(&self.save) {
      unsafe {
        button.set_disabled(false);
        button.set_focus_mode(Control::FOCUS_ALL);
      }
    }
  }

  fn connect_item_changed(&self, owner: Node, tree: &SkillTree) {
    owner.connect_to(
      match tree {
        SkillTree::Adventurer(path) => (path),
        SkillTree::Producer(path) => (path),
      },
      "item_edited",
      "item_changed",
    );
  }

  fn populate_tree(&self, owner: Node, tree: &SkillTree) {
    let (mut tree, csv) = match tree {
      SkillTree::Adventurer(path) => (
        some!(owner.get_node_as::<Tree>(path)),
        include_str!("res/adventurer_skills.csv"),
      ),
      SkillTree::Producer(path) => (
        some!(owner.get_node_as::<Tree>(path)),
        include_str!("res/producer_skills.csv"),
      ),
    };
    let skill_color = Color::rgb(0.4, 0.6, 0.7);
    let info_color = Color::rgb(0.5, 0.5, 0.5);

    unsafe {
      tree.clear();
      tree.set_focus_mode(Control::FOCUS_NONE);
      tree.set_column_expand(0, true);
      tree.set_column_min_width(0, 3);

      if let Some(parent) = tree.create_item(None, -1) {
        tree.set_focus_mode(Control::FOCUS_ALL);
        for line in csv.lines() {
          let mut iter = line.split(',');
          let skill = if let Some(text) = iter.next() {
            text
          } else {
            continue;
          };

          let mul = if let Some(text) = iter.next() {
            text
          } else {
            continue;
          };

          let id = if let Some(text) = iter.next() {
            if text.parse::<u32>().is_err() {
              continue;
            }
            text
          } else {
            continue;
          };

          if let Some(mut item) = tree.create_item(parent.cast::<Object>(), -1) {
            // Skill name.
            item.set_custom_color(0, skill_color);
            item.set_text(0, GodotString::from_str(skill));

            // Experience multiplier.
            item.set_custom_color(1, info_color);
            item.set_text(1, GodotString::from_str(&format!("{}x", mul)));

            // Skill ID.
            item.set_custom_color(2, info_color);
            item.set_text(2, GodotString::from_str(id));

            // Skill level.
            item.set_cell_mode(3, TreeItem::CELL_MODE_RANGE);
            item.set_range_config(3, 0.0, 200.0, 1.0, false);
            item.set_editable(3, true);
          }
        }
      }
    }
  }
}
