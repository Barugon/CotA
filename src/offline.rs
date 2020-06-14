use crate::constants::*;
use crate::util::*;
use gdnative::*;
use std::{cell::RefCell, path::Path};

enum SkillTree {
  Adventurer(NodePath),
  Producer(NodePath),
}

enum Confirmation {
  Load,
  Quit,
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Offline {
  info: RefCell<Option<GameInfo>>,
  confirmation: RefCell<Confirmation>,
  load: NodePath,
  save: NodePath,
  gold: NodePath,
  adv_lvl: NodePath,
  adventurer: SkillTree,
  producer: SkillTree,
  file_dialog: NodePath,
  file_dialog_title: GodotString,
  file_filters: StringArray,
  status: NodePath,
  confirm: NodePath,
  popup_centered: GodotString,
}

#[methods]
impl Offline {
  fn _init(_owner: Node) -> Self {
    let mut filters = StringArray::new();
    filters.push(&GodotString::from_str("*.sota; Saved Games"));
    Offline {
      info: RefCell::new(None),
      confirmation: RefCell::new(Confirmation::Load),
      load: NodePath::from_str("HBox/LoadButton"),
      save: NodePath::from_str("HBox/SaveButton"),
      gold: NodePath::from_str("HBox/GoldSpinBox"),
      adv_lvl: NodePath::from_str("HBox/AdvLvlSpinBox"),
      adventurer: SkillTree::Adventurer(NodePath::from_str("AdvPanel/Tree")),
      producer: SkillTree::Producer(NodePath::from_str("ProPanel/Tree")),
      file_dialog: NodePath::from_str("/root/App/FileDialog"),
      file_dialog_title: GodotString::from_str("Select Saved Game"),
      file_filters: filters,
      status: NodePath::from_str("Label"),
      confirm: NodePath::from_str("/root/App/ConfirmationDialog"),
      popup_centered: GodotString::from_str("popup_centered"),
    }
  }

  #[export]
  fn _ready(&self, owner: Node) {
    self.connect_skill_changed(owner, &self.adventurer);
    self.connect_skill_changed(owner, &self.producer);
    self.connect_spin_changed(owner, &self.gold);
    self.connect_spin_changed(owner, &self.adv_lvl);

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

    // Connect the quit dialog.
    owner.connect_to(&self.confirm, "confirmed", "confirmed");

    self.initialize_tree(owner, &self.adventurer);
    self.initialize_tree(owner, &self.producer);
  }

  #[export]
  fn _notification(&self, owner: Node, what: i64) {
    if what != MainLoop::NOTIFICATION_WM_QUIT_REQUEST {
      return;
    }

    if let Some(button) = owner.get_node_as::<Button>(&self.save) {
      unsafe {
        if !button.is_disabled() {
          if let Some(mut dialog) = owner.get_node_as::<ConfirmationDialog>(&self.confirm) {
            *self.confirmation.borrow_mut() = Confirmation::Quit;
            // Calling popup_centered on ConfirmationDialog from here causes an internal godot error.
            dialog.call_deferred(
              self.popup_centered.new_ref(),
              &[Variant::from_vector2(&Vector2::zero())],
            );
            return;
          }
        }
      }
    }
    self.quit(owner);
  }

  #[export]
  fn confirmed(&self, owner: Node) {
    match *self.confirmation.borrow() {
      Confirmation::Load => self.load(owner),
      Confirmation::Quit => self.quit(owner),
    }
  }

  #[export]
  fn skill_changed(&self, owner: Node) {
    if self.info.borrow().is_some() {
      // A skill has changed, enable the save button.
      self.enable_save(owner, true);
    }
  }

  #[export]
  fn spin_value_changed(&self, owner: Node, _val: f64) {
    if self.info.borrow().is_some() {
      // Gold or adv lvl has changed, enable the save button.
      self.enable_save(owner, true);
    }
  }

  #[export]
  fn spin_text_changed(&self, owner: Node, _text: GodotString) {
    if self.info.borrow().is_some() {
      // Gold or adv lvl has changed, enable the save button.
      self.enable_save(owner, true);
    }
  }

  #[export]
  fn load_clicked(&self, owner: Node) {
    if let Some(button) = owner.get_node_as::<Button>(&self.save) {
      unsafe {
        if !button.is_disabled() {
          if let Some(mut dialog) = owner.get_node_as::<ConfirmationDialog>(&self.confirm) {
            *self.confirmation.borrow_mut() = Confirmation::Load;
            dialog.popup_centered(Vector2::zero());
            return;
          }
        }
      }
    }
    self.load(owner);
  }

  #[export]
  fn file_selected(&self, owner: Node, path: GodotString) {
    // Clear and disable the trees.
    self.disable_tree(owner, &self.adventurer);
    self.disable_tree(owner, &self.producer);

    // Disable the gold input.
    self.enable_gold(owner, None);

    // Disable the adv lvl input.
    self.enable_adv_lvl(owner, None);

    let utf8 = path.to_utf8();
    let path_str = utf8.as_str();
    let game_info = GameInfo::read(path_str);
    if let Some(char_info) = CharInfo::new(game_info.as_ref()) {
      if self.populate_tree(owner, &self.adventurer, &char_info) {
        if self.populate_tree(owner, &self.producer, &char_info) {
          if let Some(gold) = char_info.get_gold() {
            self.enable_gold(owner, Some(gold));
            if let Some(lvl) = char_info.get_adv_lvl() {
              self.enable_adv_lvl(owner, Some(lvl));
              if let Some(path) = Path::new(path_str).file_name() {
                if let Some(path) = path.to_str() {
                  self.set_status_message(owner, &format!("Editing '{}'", path));
                }
              }
              *self.info.borrow_mut() = game_info;
              self.enable_save(owner, false);
              return;
            }
            self.enable_gold(owner, None);
          }
        }
        self.disable_tree(owner, &self.producer);
      }
      self.disable_tree(owner, &self.adventurer);
    }

    self.enable_save(owner, false);
    if let Some(path) = Path::new(path_str).file_name() {
      if let Some(path) = path.to_str() {
        self.set_status_message(owner, &format!("Unable to edit '{}'", path));
      }
    }
  }

  #[export]
  fn save_clicked(&self, owner: Node) {
    if self.save(owner) {
      return;
    }

    let info = self.info.borrow();
    let info = some!(info.as_ref());
    let path = some!(Path::new(info.path()).file_name());
    let path = some!(path.to_str());
    self.set_status_message(owner, &format!("Unable to save '{}'", path));
  }

  fn save(&self, owner: Node) -> bool {
    let mut char_info = some!(self.create_char_info(), false);
    if !self.collect_skills(owner, &self.adventurer, &mut char_info)
      || !self.collect_skills(owner, &self.producer, &mut char_info)
    {
      return false;
    }

    if let Some(spin_box) = owner.get_node_as::<SpinBox>(&self.gold) {
      let gold = unsafe { spin_box.get_value() } as i64;
      char_info.set_gold(gold);
    }

    if let Some(spin_box) = owner.get_node_as::<SpinBox>(&self.adv_lvl) {
      let lvl = unsafe { spin_box.get_value() } as u32;
      char_info.set_adv_lvl(lvl);
    }

    let mut info = self.info.borrow_mut();
    let info = some!(info.as_mut(), false);
    let json = some!(char_info.get_gold_json(), false);
    if !info.set_node_json("UserGold", &json.to_utf8().as_str()) {
      return false;
    }

    let json = some!(char_info.get_char_json(), false);
    if !info.set_node_json("CharacterSheet", &json.to_utf8().as_str()) {
      return false;
    }

    if !info.write() {
      return false;
    }

    // Saving was good, now disable the save button.
    self.enable_save(owner, false);
    return true;
  }

  fn load(&self, owner: Node) {
    let mut dialog = some!(owner.get_node_as::<FileDialog>(&self.file_dialog));
    unsafe {
      dialog.set_title(self.file_dialog_title.new_ref());
      dialog.set_mode(FileDialog::MODE_OPEN_FILE);
      dialog.set_filters(self.file_filters.new_ref());
      if let Some(dir) = dirs::config_dir() {
        let path = dir.join("Portalarium/Shroud of the Avatar/SavedGames");
        if let Some(path) = path.to_str() {
          let path = if cfg!(target_os = "windows") {
            // Change any backslashes to forward slashes.
            path.replace('\\', "/")
          } else {
            String::from(path)
          };
          dialog.set_current_dir(GodotString::from_str(path));
        }
      }
      dialog.popup_centered(Vector2::zero());
    }
  }

  fn quit(&self, owner: Node) {
    unsafe {
      if let Some(mut scene) = owner.get_tree() {
        scene.quit(0);
      }
    }
  }

  fn create_char_info(&self) -> Option<CharInfo> {
    CharInfo::new(self.info.borrow().as_ref())
  }

  fn connect_skill_changed(&self, owner: Node, tree: &SkillTree) {
    owner.connect_to(
      match tree {
        SkillTree::Adventurer(path) => (path),
        SkillTree::Producer(path) => (path),
      },
      "item_edited",
      "skill_changed",
    );
  }

  fn connect_spin_changed(&self, owner: Node, path: &NodePath) {
    let mut spin_box = some!(owner.get_node_as::<SpinBox>(path));
    unsafe {
      if spin_box
        .connect(
          GodotString::from_str("value_changed"),
          Some(owner.to_object()),
          GodotString::from_str("spin_value_changed"),
          VariantArray::new(),
          0,
        )
        .is_ok()
      {
        if let Some(mut edit) = spin_box.get_line_edit() {
          let _ = edit.connect(
            GodotString::from_str("text_changed"),
            Some(owner.to_object()),
            GodotString::from_str("spin_text_changed"),
            VariantArray::new(),
            0,
          );
        }
      }
    }
  }

  fn enable_save(&self, owner: Node, enable: bool) {
    let mut button = some!(owner.get_node_as::<Button>(&self.save));
    unsafe {
      if enable {
        button.set_disabled(false);
        button.set_focus_mode(Control::FOCUS_ALL);
      } else {
        button.set_disabled(true);
        button.set_focus_mode(Control::FOCUS_NONE);
      }
    }
  }

  fn enable_gold(&self, owner: Node, gold: Option<i64>) {
    let mut spin_box = some!(owner.get_node_as::<SpinBox>(&self.gold));
    unsafe {
      match gold {
        Some(gold) => {
          spin_box.to_range().set_value(gold as f64);
          spin_box.set_editable(true);
          spin_box.set_focus_mode(Control::FOCUS_ALL);
          if let Some(mut edit) = spin_box.get_line_edit() {
            edit.set_focus_mode(Control::FOCUS_ALL);
          }
        }
        None => {
          spin_box.to_range().set_value(0.0);
          spin_box.set_editable(false);
          spin_box.set_focus_mode(Control::FOCUS_NONE);
          if let Some(mut edit) = spin_box.get_line_edit() {
            edit.set_focus_mode(Control::FOCUS_NONE);
          }
        }
      }
    }
  }

  fn enable_adv_lvl(&self, owner: Node, lvl: Option<u32>) {
    let mut spin_box = some!(owner.get_node_as::<SpinBox>(&self.adv_lvl));
    unsafe {
      match lvl {
        Some(lvl) => {
          spin_box.to_range().set_value(lvl as f64);
          spin_box.set_editable(true);
          spin_box.set_focus_mode(Control::FOCUS_ALL);
          if let Some(mut edit) = spin_box.get_line_edit() {
            edit.set_focus_mode(Control::FOCUS_ALL);
          }
        }
        None => {
          spin_box.to_range().set_value(0.0);
          spin_box.set_editable(false);
          spin_box.set_focus_mode(Control::FOCUS_NONE);
          if let Some(mut edit) = spin_box.get_line_edit() {
            edit.set_focus_mode(Control::FOCUS_NONE);
          }
        }
      }
    }
  }

  fn initialize_tree(&self, owner: Node, tree: &SkillTree) {
    let mut tree = match tree {
      SkillTree::Adventurer(path) => some!(owner.get_node_as::<Tree>(path)),
      SkillTree::Producer(path) => some!(owner.get_node_as::<Tree>(path)),
    };
    unsafe {
      tree.set_column_expand(0, true);
      tree.set_column_min_width(0, 3);
      // tree.set_column_title(0, GodotString::from_str("Skill"));
      // tree.set_column_title(1, GodotString::from_str("Mul"));
      // tree.set_column_title(2, GodotString::from_str("ID"));
      // tree.set_column_title(3, GodotString::from_str("Level"));
      // tree.set_column_titles_visible(true);
    }
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

  fn populate_tree(&self, owner: Node, tree: &SkillTree, info: &CharInfo) -> bool {
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
    let info_color = Color::rgb(0.4, 0.4, 0.4);

    unsafe {
      let parent = some!(tree.create_item(None, -1), false);
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
          GodotString::from(text)
        } else {
          continue;
        };

        let level = if let Some(val) = info.get_skill_exp(&id) {
          let val = val as f64;
          let mut level = 0;
          // Find the level for the given experience.
          for (lvl, exp) in SKILL_EXP_VALUES.iter().enumerate().rev() {
            if val >= *exp as f64 * mul_val {
              level = lvl + 1;
              break;
            }
          }
          level
        } else {
          0
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
          item.set_text(2, id);

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

  fn collect_skills(&self, owner: Node, tree: &SkillTree, info: &mut CharInfo) -> bool {
    let mut tree = match tree {
      SkillTree::Adventurer(path) => some!(owner.get_node_as::<Tree>(path), false),
      SkillTree::Producer(path) => some!(owner.get_node_as::<Tree>(path), false),
    };
    unsafe {
      let mut root = some!(tree.get_root(), false);
      let mut node = root.get_children();
      loop {
        if let Some(mut item) = node {
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
              info.set_skill_exp(&key, exp);
            } else {
              // The level is zero, remove the skill if it exists.
              info.remove_skill(&key);
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
