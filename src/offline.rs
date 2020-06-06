use crate::constants::*;
use crate::util::*;
use gdnative::*;
use std::{fs::File, io::prelude::*, path::Path};
use xml_dom::*;

enum SkillTree {
  Adventurer(NodePath),
  Producer(NodePath),
}

struct GameInfo {
  node: level2::RefNode,
  path: GodotString,
}

impl GameInfo {
  fn get_node_json(&self, name: &str) -> Option<String> {
    if let Ok(document) = level2::convert::as_document(&self.node) {
      let collections = document.get_elements_by_tag_name("collection");
      for collection in collections {
        if let Ok(element) = level2::convert::as_element(&collection) {
          if let Some(attribute) = element.get_attribute("name") {
            if attribute == name {
              let records = element.get_elements_by_tag_name("record");
              for record in records {
                if let Ok(element) = level2::convert::as_element(&record) {
                  let nodes = element.child_nodes();
                  for node in nodes {
                    if let Ok(text) = level2::convert::as_text(&node) {
                      return text.node_value();
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
    None
  }

  fn set_node_json(&mut self, name: &str, json: &str) -> bool {
    if let Ok(document) = level2::convert::as_document_mut(&mut self.node) {
      let mut collections = document.get_elements_by_tag_name("collection");
      for collection in &mut collections {
        if let Ok(element) = level2::convert::as_element_mut(collection) {
          if let Some(attribute) = element.get_attribute("name") {
            if attribute == name {
              let mut records = element.get_elements_by_tag_name("record");
              for record in &mut records {
                if let Ok(element) = level2::convert::as_element_mut(record) {
                  let mut nodes = element.child_nodes();
                  for node in &mut nodes {
                    if let Ok(text) = level2::convert::as_text_mut(node) {
                      return text.set_node_value(json).is_ok();
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
    false
  }
}

struct CharacterInfo {
  char_json: json::JsonValue,
  gold_json: json::JsonValue,
  date: String,
}

impl CharacterInfo {
  fn new(info: &Option<GameInfo>) -> Option<Self> {
    if let Some(info) = info {
      // Get the 'CharacterSheet' json.
      if let Some(text) = info.get_node_json("CharacterSheet") {
        if let Ok(char_json) = json::parse(&text) {
          // Get the date.
          if let Some(date) = char_json["rd"]["c"].as_str() {
            // Ge the 'UserGold' json.
            if let Some(text) = info.get_node_json("UserGold") {
              if let Ok(gold_json) = json::parse(&text) {
                let date = String::from(date);
                return Some(Self {
                  char_json,
                  gold_json,
                  date,
                });
              }
            }
          }
        }
      }
    }
    None
  }
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Offline {
  info: Option<GameInfo>,
  load: NodePath,
  save: NodePath,
  gold: NodePath,
  adventurer: SkillTree,
  producer: SkillTree,
  file_dialog: NodePath,
  file_dialog_title: GodotString,
  file_filters: StringArray,
  status: NodePath,
}

#[methods]
impl Offline {
  fn _init(_owner: Node) -> Self {
    let mut filters = StringArray::new();
    filters.push(&GodotString::from_str("*.sota; Saved Games"));
    Offline {
      info: None,
      load: NodePath::from_str("HBox/LoadButton"),
      save: NodePath::from_str("HBox/SaveButton"),
      gold: NodePath::from_str("HBox/SpinBox"),
      adventurer: SkillTree::Adventurer(NodePath::from_str("AdvPanel/Tree")),
      producer: SkillTree::Producer(NodePath::from_str("ProPanel/Tree")),
      file_dialog: NodePath::from_str("/root/App/FileDialog"),
      file_dialog_title: GodotString::from_str("Select Saved Game"),
      file_filters: filters,
      status: NodePath::from_str("Label"),
    }
  }

  #[export]
  fn _ready(&self, owner: Node) {
    self.connect_item_changed(owner, &self.adventurer);
    self.connect_item_changed(owner, &self.producer);

    // Connect load button.
    owner.connect_to(&self.load, "pressed", "load_clicked");

    // Connect file_selected.
    owner.connect_to(&self.file_dialog, "file_selected", "file_selected");

    // Connect save_clicked.
    owner.connect_to(&self.save, "pressed", "save_clicked");
  }

  #[export]
  fn item_changed(&self, owner: Node) {
    if self.info.is_some() {
      if let Some(mut button) = owner.get_node_as::<Button>(&self.save) {
        unsafe {
          button.set_disabled(false);
          button.set_focus_mode(Control::FOCUS_ALL);
        }
      }
    }
  }

  #[export]
  fn load_clicked(&self, owner: Node) {
    if let Some(mut dialog) = owner.get_node_as::<FileDialog>(&self.file_dialog) {
      unsafe {
        dialog.set_title(self.file_dialog_title.new_ref());
        dialog.set_mode(FileDialog::MODE_OPEN_FILE);
        dialog.set_filters(self.file_filters.new_ref());
        if let Some(dir) = dirs::config_dir() {
          let path = dir.join("Portalarium/Shroud of the Avatar/SavedGames");
          if let Some(path) = path.to_str() {
            dialog.set_current_dir(GodotString::from_str(path));
          }
        }
        dialog.popup_centered(Vector2::zero());
      }
    }
  }

  #[export]
  fn file_selected(&mut self, owner: Node, path: GodotString) {
    let utf8 = path.to_utf8();
    let path_str = utf8.as_str();
    if let Ok(text) = std::fs::read_to_string(path_str) {
      if let Ok(node) = parser::read_xml(&text) {
        let game_info = Some(GameInfo {
          node,
          path: path.new_ref(),
        });
        if let Some(char_info) = CharacterInfo::new(&game_info) {
          let json = &char_info.char_json["sk2"];
          if json.is_object() {
            if self.populate_tree(owner, &self.adventurer, json)
              && self.populate_tree(owner, &self.producer, json)
            {
              if let Some(path) = Path::new(path_str).file_name() {
                if let Some(path) = path.to_str() {
                  self.set_status_message(owner, &format!("Editing '{}'", path));
                }
              }
              self.info = game_info;
              return;
            }
          }
        }
      }
    }
    // Failure to edit. Cleanup
    self.disable_tree(owner, &self.adventurer);
    self.disable_tree(owner, &self.producer);

    if let Some(mut button) = owner.get_node_as::<Button>(&self.save) {
      unsafe {
        button.set_disabled(true);
        button.set_focus_mode(Control::FOCUS_NONE);
      }
    }

    if let Some(mut gold) = owner.get_node_as::<SpinBox>(&self.gold) {
      unsafe {
        gold.set_value(0.0);
        gold.set_editable(false);
        gold.set_focus_mode(Control::FOCUS_NONE);
      }
    }

    if let Some(path) = Path::new(path_str).file_name() {
      if let Some(path) = path.to_str() {
        self.set_status_message(owner, &format!("Unable to edit '{}'", path));
      }
    }
  }

  #[export]
  fn save_clicked(&mut self, owner: Node) {
    if let Some(mut char_info) = CharacterInfo::new(&self.info) {
      let sk2 = &mut char_info.char_json["sk2"];
      if sk2.is_object() {
        if self.collect_skills(owner, &self.adventurer, sk2, &char_info.date)
          && self.collect_skills(owner, &self.producer, sk2, &char_info.date)
        {
          if let Some(info) = &mut self.info {
            if info.set_node_json("CharacterSheet", &char_info.char_json.to_string()) {
              if let Ok(mut file) = File::create(info.path.to_utf8().as_str()) {
                if file.write_all(info.node.to_string().as_bytes()).is_ok() {
                  if let Some(mut button) = owner.get_node_as::<Button>(&self.save) {
                    unsafe {
                      button.set_disabled(true);
                      button.set_focus_mode(Control::FOCUS_NONE);
                    }
                  }
                  return;
                }
              }
            }
          }
        }
      }
    }

    if let Some(info) = &self.info {
      if let Some(path) = Path::new(info.path.to_utf8().as_str()).file_name() {
        if let Some(path) = path.to_str() {
          self.set_status_message(owner, &format!("Unable to save '{}'", path));
        }
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

  fn disable_tree(&self, owner: Node, tree: &SkillTree) {
    let mut tree = match tree {
      SkillTree::Adventurer(path) => some!(owner.get_node_as::<Tree>(path)),
      SkillTree::Producer(path) => some!(owner.get_node_as::<Tree>(path)),
    };
    unsafe {
      tree.clear();
      tree.set_focus_mode(Control::FOCUS_NONE);
    }
  }

  fn populate_tree(&self, owner: Node, tree: &SkillTree, json: &json::JsonValue) -> bool {
    let (mut tree, csv) = match tree {
      SkillTree::Adventurer(path) => (
        some!(owner.get_node_as::<Tree>(path), false),
        ADVENTURER_SKILLS,
      ),
      SkillTree::Producer(path) => (
        some!(owner.get_node_as::<Tree>(path), false),
        PRODUCER_SKILLS,
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

          let mul_val = if let Ok(val) = mul.parse::<f64>() {
            val
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

            let level = if let Some(val) = json[id]["x"].as_f64() {
              let mut level = 0;
              for (lvl, exp) in EXP_VALUES.iter().enumerate().rev() {
                if val >= *exp as f64 * mul_val {
                  level = lvl + 1;
                  break;
                }
              }
              level
            } else {
              0
            };

            // Skill level.
            item.set_cell_mode(3, TreeItem::CELL_MODE_RANGE);
            item.set_range_config(3, 0.0, 200.0, 1.0, false);
            item.set_range(3, level as f64);
            item.set_editable(3, true);
          }
        }
        return true;
      }
    }
    false
  }

  fn collect_skills(
    &self,
    owner: Node,
    tree: &SkillTree,
    json: &mut json::JsonValue,
    date: &str,
  ) -> bool {
    let mut tree = match tree {
      SkillTree::Adventurer(path) => some!(owner.get_node_as::<Tree>(path), false),
      SkillTree::Producer(path) => some!(owner.get_node_as::<Tree>(path), false),
    };
    unsafe {
      let mut root = some!(tree.get_root(), false);
      let mut node = root.get_children();
      loop {
        if let Some(mut item) = node {
          if let Ok(mul) = item
            .get_text(1)
            .to_utf8()
            .as_str()
            .trim_end_matches('x')
            .parse::<f64>()
          {
            let utf8 = item.get_text(2).to_utf8();
            let key = utf8.as_str();
            let lvl = item.get_range(3) as usize;
            if lvl > 0 {
              let exp = (EXP_VALUES[lvl - 1] as f64 * mul).ceil() as i64;
              if let Some(cur) = json[key]["x"].as_i64() {
                // Change it only if it's different.
                if exp != cur {
                  json[key]["x"] = json::JsonValue::from(exp);
                }
              } else {
                // Add a new object for the skill ID.
                json[key] = json::object! {
                  "x": exp,
                  "t": date,
                  "m": 0
                };
              }
            } else {
              // Remove the skill if it exists.
              json.remove(key);
            }
          }
          node = item.get_next();
        } else {
          return true;
        }
      }
    }
  }

  fn set_status_message(&self, owner: Node, text: &str) {
    if let Some(mut label) = owner.get_node_as::<Label>(&self.status) {
      unsafe {
        label.set_text(GodotString::from_str(text));
      }
    }
  }
}
