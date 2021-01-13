use crate::constants::*;
use crate::util::*;
use gdnative::api::*;
use gdnative::prelude::*;
use std::{cell::RefCell, fs::File, io::prelude::*, path::Path};

const USER_ID: &str = "000000000000000000000001";

enum SkillTree {
  Adventurer,
  Producer,
}

enum Confirmation {
  Load,
  Quit,
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Offline {
  game_info: RefCell<Option<GameInfo>>,
  confirmation: RefCell<Confirmation>,
  load: GodotString,
  save: GodotString,
  gold: GodotString,
  adv_lvl: GodotString,
  adventurer: GodotString,
  producer: GodotString,
  file_dialog: GodotString,
  file_dialog_title: GodotString,
  file_filters: StringArray,
  status: GodotString,
  confirm: GodotString,
  popup_centered: GodotString,
}

#[methods]
impl Offline {
  fn new(_owner: &Node) -> Self {
    let mut filters = StringArray::new();
    filters.push(GodotString::from("*.sota; Saved Games"));
    Offline {
      game_info: RefCell::new(None),
      confirmation: RefCell::new(Confirmation::Load),
      load: GodotString::from("HBox/LoadButton"),
      save: GodotString::from("HBox/SaveButton"),
      gold: GodotString::from("HBox/GoldSpinBox"),
      adv_lvl: GodotString::from("HBox/AdvLvlSpinBox"),
      adventurer: GodotString::from("AdvPanel/Tree"),
      producer: GodotString::from("ProPanel/Tree"),
      file_dialog: GodotString::from("/root/App/FileDialog"),
      file_dialog_title: GodotString::from("Select Saved Game"),
      file_filters: filters,
      status: GodotString::from("Label"),
      confirm: GodotString::from("/root/App/ConfirmationDialog"),
      popup_centered: GodotString::from("popup_centered"),
    }
  }

  #[export]
  fn _ready(&self, owner: TRef<Node>) {
    owner.connect_to(&self.adventurer, "item_edited", "adv_skill_changed");
    owner.connect_to(&self.producer, "item_edited", "pro_skill_changed");
    owner.connect_to(&self.adv_lvl, "value_changed", "spin_value_changed");
    owner.connect_to(&self.gold, "value_changed", "spin_value_changed");

    // Make the edit portion of the gold entry unfocusable.
    self.enable_gold(owner, None);

    // Make the edit portion of the adv lvl entry unfocusable.
    self.enable_adv_lvl(owner, None);

    // Connect load button.
    owner.connect_to(&self.load, "pressed", "load_clicked");

    // Connect file_selected.
    owner.connect_to(&self.file_dialog, "file_selected", "file_selected");

    // Connect save_clicked.
    owner.connect_to(&self.save, "pressed", "save_clicked");

    // Connect the confirmation dialog.
    owner.connect_to(&self.confirm, "confirmed", "confirmed");

    self.initialize_tree(owner, SkillTree::Adventurer);
    self.initialize_tree(owner, SkillTree::Producer);
  }

  #[export]
  fn _notification(&self, owner: TRef<Node>, what: i64) {
    if what != MainLoop::NOTIFICATION_WM_QUIT_REQUEST {
      return;
    }

    if let Some(button) = owner.get_node_as::<Button>(&self.save) {
      if !button.is_disabled() {
        *self.confirmation.borrow_mut() = Confirmation::Quit;
        // Calling popup_centered directly from here causes an internal godot error.
        owner.method_deferred(
          &self.confirm,
          &self.popup_centered,
          &[Variant::from_vector2(&Vector2::zero())],
        );
        return;
      }
    }
    self.quit(owner);
  }

  #[export]
  fn confirmed(&self, owner: TRef<Node>) {
    match *self.confirmation.borrow() {
      Confirmation::Load => self.load(owner),
      Confirmation::Quit => self.quit(owner),
    }
  }

  #[export]
  fn adv_skill_changed(&self, owner: TRef<Node>) {
    self.skill_changed(owner, SkillTree::Adventurer);
  }

  #[export]
  fn pro_skill_changed(&self, owner: TRef<Node>) {
    self.skill_changed(owner, SkillTree::Producer);
  }

  #[export]
  fn spin_value_changed(&self, owner: TRef<Node>, _val: f64) {
    if self.game_info.borrow().is_some() {
      // Gold or adv lvl has changed, enable the save button.
      self.enable_save(owner, true);
    }
  }

  #[export]
  fn load_clicked(&self, owner: TRef<Node>) {
    if let Some(button) = owner.get_node_as::<Button>(&self.save) {
      if !button.is_disabled() {
        if let Some(dialog) = owner.get_node_as::<ConfirmationDialog>(&self.confirm) {
          *self.confirmation.borrow_mut() = Confirmation::Load;
          dialog.popup_centered(Vector2::zero());
          return;
        }
      }
    }
    self.load(owner);
  }

  #[export]
  fn file_selected(&self, owner: TRef<Node>, path: GodotString) {
    // Clear and disable the trees.
    self.disable_tree(owner, SkillTree::Adventurer);
    self.disable_tree(owner, SkillTree::Producer);

    // Disable the gold input.
    self.enable_gold(owner, None);

    // Disable the adv lvl input.
    self.enable_adv_lvl(owner, None);

    if let Some(game_info) = GameInfo::load(&path) {
      if self.populate_tree(owner, SkillTree::Adventurer, &game_info) {
        if self.populate_tree(owner, SkillTree::Producer, &game_info) {
          if let Some(gold) = game_info.get_gold() {
            self.enable_gold(owner, Some(gold));
            if let Some(lvl) = game_info.get_adv_lvl() {
              self.enable_adv_lvl(owner, Some(lvl));
              *self.game_info.borrow_mut() = Some(game_info);

              // Set the status message.
              let path = path.to_utf8();
              if let Some(name) = Path::new(path.as_str()).file_name() {
                if let Some(name) = name.to_str() {
                  self.set_status_message(owner, &format!(r#"Editing "{}""#, name));
                }
              }

              // Disable the save button.
              self.enable_save(owner, false);
              return;
            }
            self.enable_gold(owner, None);
          }
        }
        self.disable_tree(owner, SkillTree::Producer);
      }
      self.disable_tree(owner, SkillTree::Adventurer);
    }

    // Disable the save button.
    self.enable_save(owner, false);

    // Set the status message.
    let path = path.to_utf8();
    let name = some!(Path::new(path.as_str()).file_name());
    let name = some!(name.to_str());
    self.set_status_message(owner, &format!(r#"Unable to edit "{}""#, name));
  }

  #[export]
  fn save_clicked(&self, owner: TRef<Node>) {
    if self.save(owner) {
      return;
    }

    let game_info = self.game_info.borrow();
    let game_info = some!(game_info.as_ref());
    let path = game_info.path().to_utf8();
    let name = some!(Path::new(path.as_str()).file_name());
    let name = some!(name.to_str());
    self.set_status_message(owner, &format!(r#"Unable to save "{}""#, name));
  }

  fn skill_changed(&self, owner: TRef<Node>, tree: SkillTree) {
    if let Some(info) = self.game_info.borrow().as_ref() {
      let tree = match tree {
        SkillTree::Adventurer => &self.adventurer,
        SkillTree::Producer => &self.producer,
      };
      let tree = some!(owner.get_node_as::<Tree>(tree));

      // Make sure the value actually changed.
      let item = some!(tree.get_edited());
      let item = item.to_ref();
      if let Ok(mul) = item
        .get_text(1)
        .to_utf8()
        .as_str()
        .trim_end_matches('x')
        .parse::<f64>()
      {
        let cur = info.get_skill_exp(&item.get_text(2));
        let lvl = item.get_range(3) as usize;
        if lvl > 0 {
          let exp = (SKILL_EXP_VALUES[lvl - 1] as f64 * mul).ceil() as i64;
          if let Some(cur) = cur {
            if cur == exp {
              // No change.
              return;
            }
          }
        } else if cur.is_none() {
          return;
        }
        self.enable_save(owner, true);
      }
    }
  }

  fn save(&self, owner: TRef<Node>) -> bool {
    let mut game_info = self.game_info.borrow_mut();
    let game_info = some!(game_info.as_mut(), false);
    if !self.collect_skills(owner, SkillTree::Adventurer, game_info)
      || !self.collect_skills(owner, SkillTree::Producer, game_info)
    {
      return false;
    }

    if let Some(spin_box) = owner.get_node_as::<SpinBox>(&self.gold) {
      let gold = spin_box.value() as i64;
      game_info.set_gold(gold);
    }

    if let Some(spin_box) = owner.get_node_as::<SpinBox>(&self.adv_lvl) {
      let lvl = spin_box.value() as u32;
      game_info.set_adv_lvl(lvl);
    }

    if !game_info.save() {
      return false;
    }

    // Saving was good, now disable the save button.
    self.enable_save(owner, false);
    true
  }

  fn load(&self, owner: TRef<Node>) {
    let dialog = some!(owner.get_node_as::<FileDialog>(&self.file_dialog));

    dialog.set_title(self.file_dialog_title.clone());
    dialog.set_mode(FileDialog::MODE_OPEN_FILE);
    dialog.set_filters(self.file_filters.clone());
    if let Some(dir) = dirs::config_dir() {
      let path = dir.join("Portalarium/Shroud of the Avatar/SavedGames");
      if let Some(path) = path.to_str() {
        let path = if cfg!(target_os = "windows") {
          // Change any backslashes to forward slashes.
          GodotString::from(path.replace('\\', "/"))
        } else {
          GodotString::from(path)
        };
        dialog.set_current_dir(path);
      }
    }
    dialog.popup_centered(Vector2::zero());
  }

  fn quit(&self, owner: TRef<Node>) {
    if let Some(scene) = owner.get_tree() {
      scene.to_ref().quit(0);
    }
  }

  fn enable_save(&self, owner: TRef<Node>, enable: bool) {
    let button = some!(owner.get_node_as::<Button>(&self.save));

    if enable {
      button.set_disabled(false);
      button.set_focus_mode(Control::FOCUS_ALL);
    } else {
      button.set_disabled(true);
      button.set_focus_mode(Control::FOCUS_NONE);
    }
  }

  fn enable_gold(&self, owner: TRef<Node>, gold: Option<i64>) {
    let spin_box = some!(owner.get_node_as::<SpinBox>(&self.gold));

    match gold {
      Some(gold) => {
        spin_box.set_value(gold as f64);
        spin_box.set_editable(true);
        spin_box.set_focus_mode(Control::FOCUS_ALL);
        if let Some(edit) = spin_box.get_line_edit() {
          edit.to_ref().set_focus_mode(Control::FOCUS_ALL);
        }
      }
      None => {
        spin_box.set_value(0.0);
        spin_box.set_editable(false);
        spin_box.set_focus_mode(Control::FOCUS_NONE);
        if let Some(edit) = spin_box.get_line_edit() {
          edit.to_ref().set_focus_mode(Control::FOCUS_NONE);
        }
      }
    }
  }

  fn enable_adv_lvl(&self, owner: TRef<Node>, lvl: Option<u32>) {
    let spin_box = some!(owner.get_node_as::<SpinBox>(&self.adv_lvl));

    match lvl {
      Some(lvl) => {
        spin_box.set_value(lvl as f64);
        spin_box.set_editable(true);
        spin_box.set_focus_mode(Control::FOCUS_ALL);
        if let Some(edit) = spin_box.get_line_edit() {
          edit.to_ref().set_focus_mode(Control::FOCUS_ALL);
        }
      }
      None => {
        spin_box.set_value(0.0);
        spin_box.set_editable(false);
        spin_box.set_focus_mode(Control::FOCUS_NONE);
        if let Some(edit) = spin_box.get_line_edit() {
          edit.to_ref().set_focus_mode(Control::FOCUS_NONE);
        }
      }
    }
  }

  fn initialize_tree(&self, owner: TRef<Node>, tree: SkillTree) {
    let tree = match tree {
      SkillTree::Adventurer => some!(owner.get_node_as::<Tree>(&self.adventurer)),
      SkillTree::Producer => some!(owner.get_node_as::<Tree>(&self.producer)),
    };

    tree.set_column_expand(0, true);
    tree.set_column_min_width(0, 3);
    // tree.set_column_title(0, GodotString::from("Skill"));
    // tree.set_column_title(1, GodotString::from("Mul"));
    // tree.set_column_title(2, GodotString::from("ID"));
    // tree.set_column_title(3, GodotString::from("Level"));
    // tree.set_column_titles_visible(true);
  }

  fn disable_tree(&self, owner: TRef<Node>, tree: SkillTree) {
    let tree = match tree {
      SkillTree::Adventurer => some!(owner.get_node_as::<Tree>(&self.adventurer)),
      SkillTree::Producer => some!(owner.get_node_as::<Tree>(&self.producer)),
    };

    tree.clear();
    tree.set_focus_mode(Control::FOCUS_NONE);
  }

  fn populate_tree(&self, owner: TRef<Node>, tree: SkillTree, game_info: &GameInfo) -> bool {
    let (tree, csv) = match tree {
      SkillTree::Adventurer => (
        some!(owner.get_node_as::<Tree>(&self.adventurer), false),
        ADVENTURER_SKILLS,
      ),
      SkillTree::Producer => (
        some!(owner.get_node_as::<Tree>(&self.producer), false),
        PRODUCER_SKILLS,
      ),
    };

    let skill_color = Color::rgb(0.4, 0.6, 0.7);
    let info_color = Color::rgb(0.4, 0.4, 0.4);
    let root = some!(tree.create_item(Object::null(), -1), false);
    let mut parent = None;
    let mut category = "";

    tree.set_focus_mode(Control::FOCUS_ALL);
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
            item.set_selectable(3, false);
            item.set_text(0, GodotString::from(text));
            item.set_collapsed(true);
            parent = Some(item);
          }
        }

        if let Some(parent) = parent {
          // Get the skill name.
          let skill = if let Some(text) = iter.next() {
            text
          } else {
            continue;
          };

          // Get the skill multiplier text.
          let mul_text = if let Some(text) = iter.next() {
            text
          } else {
            continue;
          };

          // Parse the multiplier text.
          let mul = if let Ok(val) = mul_text.parse::<f64>() {
            val
          } else {
            continue;
          };

          // Get the skill ID.
          let id = if let Some(text) = iter.next() {
            if text.parse::<u32>().is_err() {
              continue;
            }
            GodotString::from(text)
          } else {
            continue;
          };

          // Find the skill level from the experience.
          let level = if let Some(val) = game_info.get_skill_exp(&id) {
            let val = (val as f64 / mul) as i64;
            match find_min(val, &SKILL_EXP_VALUES) {
              Some(level) => level + 1,
              None => 0,
            }
          } else {
            0
          };

          if let Some(item) = tree.create_item(parent, -1) {
            let item = item.to_ref();
            // Skill name.
            item.set_custom_color(0, skill_color);
            item.set_text(0, GodotString::from(skill));

            // Experience multiplier.
            item.set_custom_color(1, info_color);
            item.set_text(1, GodotString::from(format!("{}x", mul_text)));

            // Skill ID.
            item.set_custom_color(2, info_color);
            item.set_text(2, id);

            // Skill level.
            item.set_cell_mode(3, TreeItem::CELL_MODE_RANGE);
            item.set_range_config(3, 0.0, 200.0, 1.0, false);
            item.set_range(3, level as f64);
            item.set_editable(3, true);
          }
        }
      }
    }
    true
  }

  fn collect_skills(&self, owner: TRef<Node>, tree: SkillTree, game_info: &mut GameInfo) -> bool {
    let tree = match tree {
      SkillTree::Adventurer => some!(owner.get_node_as::<Tree>(&self.adventurer), false),
      SkillTree::Producer => some!(owner.get_node_as::<Tree>(&self.producer), false),
    };

    let root = some!(tree.get_root(), false);
    let root = root.to_ref();
    let mut parent = root.get_children();
    while let Some(item) = parent {
      let item = item.to_ref();
      let mut child = item.get_children();
      while let Some(item) = child {
        let item = item.to_ref();

        // Get the skill multiplier.
        if let Ok(mul) = item
          .get_text(1)
          .to_utf8()
          .as_str()
          .trim_end_matches('x')
          .parse::<f64>()
        {
          // Get the skill ID.
          let key = item.get_text(2);

          // Get the skill level.
          let lvl = item.get_range(3) as usize;
          if lvl > 0 {
            let exp = (SKILL_EXP_VALUES[lvl - 1] as f64 * mul).ceil() as i64;
            game_info.set_skill_exp(&key, exp);
          } else {
            // The level is zero, remove the skill if it exists.
            game_info.remove_skill(&key);
          }
        }
        child = item.get_next();
      }
      parent = item.get_next();
    }
    true
  }

  fn set_status_message(&self, owner: TRef<Node>, text: &str) {
    if let Some(label) = owner.get_node_as::<Label>(&self.status) {
      label.set_text(GodotString::from(text));
    }
  }
}

fn find_min<T: Ord>(value: T, values: &[T]) -> Option<usize> {
  match values.binary_search(&value) {
    Ok(idx) => Some(idx),
    Err(idx) => {
      if idx > 0 {
        Some(idx - 1)
      } else {
        None
      }
    }
  }
}

trait Get {
  fn get(&self, key: &Variant) -> Option<Variant>;
}

impl Get for Variant {
  fn get(&self, key: &Variant) -> Option<Variant> {
    if let Some(dictionary) = self.try_to_dictionary() {
      return Some(dictionary.get(key));
    }
    None
  }
}

impl Get for Option<Variant> {
  fn get(&self, key: &Variant) -> Option<Variant> {
    if let Some(variant) = self {
      return variant.get(key);
    }
    None
  }
}

trait Set {
  fn set(&mut self, key: &Variant, value: &Variant) -> bool;
}

impl Set for Variant {
  fn set(&mut self, key: &Variant, value: &Variant) -> bool {
    if let Some(dictionary) = self.try_to_dictionary() {
      unsafe { dictionary.assume_unique() }.insert(key, value);
      return true;
    }
    false
  }
}

impl Set for Option<Variant> {
  fn set(&mut self, key: &Variant, value: &Variant) -> bool {
    if let Some(variant) = self {
      return variant.set(key, value);
    }
    false
  }
}

trait Erase {
  fn erase(&mut self, key: &Variant);
}

impl Erase for Variant {
  fn erase(&mut self, key: &Variant) {
    if let Some(dictionary) = self.try_to_dictionary() {
      unsafe { dictionary.assume_unique() }.erase(key);
    }
  }
}

impl Erase for Option<Variant> {
  fn erase(&mut self, key: &Variant) {
    if let Some(variant) = self {
      return variant.erase(key);
    }
  }
}

trait ToText {
  fn to_text(&self) -> Option<GodotString>;
}

impl ToText for Option<Variant> {
  fn to_text(&self) -> Option<GodotString> {
    if let Some(variant) = self {
      return Some(variant.to_godot_string());
    }
    None
  }
}

trait ToInt {
  fn to_int(&self) -> Option<i64>;
}

impl ToInt for Option<Variant> {
  fn to_int(&self) -> Option<i64> {
    if let Some(variant) = self {
      return Some(variant.to_i64());
    }
    None
  }
}

// Structure to load and modify a SotA save-game file.
struct GameInfo {
  // Save file path.
  path: GodotString,
  // Avatar ID.
  toon: GodotString,
  // XML text.
  xml: String,
  // Dictionaries.
  character: Variant,
  skills: Variant,
  gold: Variant,
  // Save date.
  date: GodotString,
  // Keys.
  ae: Variant,
  g: Variant,
  m: Variant,
  t: Variant,
  x: Variant,
}

fn get_avatar_id(text: &str) -> Option<GodotString> {
  // Find the 'User' collection tag.
  let find = r#"<collection name="User">"#;
  let pos = some!(text.find(find), None);
  let text = &text[pos + find.len()..];

  // Find the record tag.
  let find = format!(r#"<record Id="{}">"#, USER_ID);
  let pos = some!(text.find(&find), None);
  let text = &text[pos + find.len()..];

  // Find the record end tag.
  let pos = some!(text.find("</record>"), None);
  let text = &text[..pos];

  // Parse the JSON text.
  let result = some!(JSON::godot_singleton().parse(GodotString::from(text)), None);
  let var = result.to_ref().result();

  // Get the avatar ID.
  let dc = Variant::from_str("dc");
  let id = some!(var.get(&dc), None);
  Some(id.to_godot_string())
}

fn get_json(text: &str, collection: &str, id: &str) -> Option<Variant> {
  // Find the collection tag.
  let find = format!(r#"<collection name="{}">"#, collection);
  let pos = some!(text.find(&find), None);
  let text = &text[pos + find.len()..];

  // From that point, find the record tag.
  let find = format!(r#"<record Id="{}">"#, id);
  let pos = some!(text.find(&find), None);
  let text = &text[pos + find.len()..];

  // Find the record end tag.
  let pos = some!(text.find("</record>"), None);
  let text = &text[..pos];

  // Parse the JSON text.
  let result = some!(JSON::godot_singleton().parse(GodotString::from(text)), None);
  let var = result.to_ref().result();
  if var.try_to_dictionary().is_some() {
    return Some(var);
  }
  None
}

fn set_json(text: &str, collection: &str, id: &str, var: &Variant) -> Option<String> {
  // Find the collection tag.
  let find = format!(r#"<collection name="{}">"#, collection);
  let start = some!(text.find(&find), None) + find.len();
  let slice = &text[start..];

  // From that point, find the record tag.
  let find = format!(r#"<record Id="{}">"#, id);
  let pos = some!(slice.find(&find), None) + find.len();
  let slice = &slice[pos..];
  let start = start + pos;

  // Find the record end tag.
  let pos = some!(slice.find("</record>"), None);
  let end = start + pos;

  // Convert the dictionary to JSON text.
  let dict = some!(var.try_to_dictionary(), None);
  let json = dict.to_json();
  let json = json.to_utf8();
  let json = json.as_str();

  // Concatenate the XML with the new JSON.
  let parts = [&text[..start], json, &text[end..]];
  let mut result = String::new();
  result.reserve(parts[0].len() + parts[1].len() + parts[2].len());
  result.push_str(parts[0]);
  result.push_str(parts[1]);
  result.push_str(parts[2]);
  Some(result)
}

fn find_date(var: &Variant) -> Option<GodotString> {
  let dict = some!(var.try_to_dictionary(), None);
  let t = Variant::from_str("t");
  for (_, var) in dict.iter() {
    if let Some(var) = var.get(&t) {
      return Some(var.to_godot_string());
    }
  }
  None
}

impl GameInfo {
  fn load(path: &GodotString) -> Option<Self> {
    match std::fs::read_to_string(path.to_utf8().as_str()) {
      Ok(xml) => {
        // Get the avatar ID.
        let msg = "Unable to determine the current avatar";
        let toon = some!(get_avatar_id(&xml), msg, None);
        let id = toon.to_utf8();
        let id = id.as_str();

        // Find the 'CharacterSheet' JSON.
        let msg = "Unable to find character sheet";
        let character = some!(get_json(&xml, "CharacterSheet", id), msg, None);

        // Get the skills dictionary.
        let msg = "Skills dictionary not found";
        let skills = some!(character.get(&Variant::from_str("sk2")), msg, None);

        // Find the 'UserGold' json.
        let msg = "Unable to find user gold";
        let gold = some!(get_json(&xml, "UserGold", USER_ID), msg, None);

        // Find a date.
        let msg = "Unable to find the date/time";
        let date = some!(find_date(&skills), msg, None);

        return Some(GameInfo {
          path: path.clone(),
          toon,
          xml,
          character,
          skills,
          gold,
          date,
          ae: Variant::from_str("ae"),
          g: Variant::from_str("g"),
          m: Variant::from_str("m"),
          t: Variant::from_str("t"),
          x: Variant::from_str("x"),
        });
      }
      Err(err) => {
        if let Some(err) = err.get_ref() {
          godot_print!("Unable to load: {:?}", err);
        }
      }
    }
    None
  }

  fn save(&self) -> bool {
    let id = self.toon.to_utf8();
    let id = id.as_str();

    // Set UserGold.
    let tag = "UserGold";
    let msg = "Unable to set UserGold";
    let xml = some!(set_json(&self.xml, tag, USER_ID, &self.gold), msg, false);

    // Set CharacterSheet.
    let tag = "CharacterSheet";
    let msg = "Unable to set CharacterSheet";
    let xml = some!(set_json(&xml, tag, id, &self.character), msg, false);

    // Create the save-game file and store the XML data.
    match File::create(self.path.to_utf8().as_str()) {
      Ok(mut file) => match file.write_all(xml.as_bytes()) {
        Ok(()) => return true,
        Err(err) => {
          if let Some(err) = err.get_ref() {
            godot_print!("Unable to save: {:?}", err);
          }
        }
      },
      Err(err) => {
        if let Some(err) = err.get_ref() {
          godot_print!("Unable to save: {:?}", err);
        }
      }
    }
    false
  }

  fn get_gold(&self) -> Option<i64> {
    self.gold.get(&self.g).to_int()
  }

  fn set_gold(&mut self, gold: i64) {
    self.gold.set(&self.g, &Variant::from_i64(gold));
  }

  fn get_adv_lvl(&self) -> Option<u32> {
    if let Some(val) = self.character.get(&self.ae).to_int() {
      if let Some(level) = find_min(val, &LEVEL_EXP_VALUES) {
        return Some(level as u32 + 1);
      }
    }
    None
  }

  fn set_adv_lvl(&mut self, lvl: u32) {
    let exp = LEVEL_EXP_VALUES[lvl as usize - 1];
    self.character.set(&self.ae, &Variant::from_i64(exp));
  }

  fn get_skill_exp(&self, key: &GodotString) -> Option<i64> {
    let var = Variant::from_godot_string(key);
    self.skills.get(&var).get(&self.x).to_int()
  }

  fn set_skill_exp(&mut self, key: &GodotString, exp: i64) {
    let key = Variant::from_godot_string(key);
    if let Some(mut dict) = self.skills.get(&key) {
      if let Some(cur) = dict.get(&self.x).to_int() {
        // Change it only if it's different.
        if exp != cur {
          dict.set(&self.x, &Variant::from_i64(exp));
          return;
        }
        return;
      }
    }

    // Add a new dictionary for the skill ID.
    let dict = Dictionary::new();
    dict.insert(&self.x, exp);
    dict.insert(&self.t, self.date.clone());
    dict.insert(&self.m, 0i64);
    let var = Variant::from_dictionary(&dict.into_shared());
    self.skills.set(&key, &var);
  }

  fn remove_skill(&mut self, key: &GodotString) {
    self.skills.erase(&Variant::from_godot_string(key));
  }

  fn path(&self) -> &GodotString {
    &self.path
  }
}
