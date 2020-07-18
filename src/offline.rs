use crate::constants::*;
use crate::util::*;
use gdnative::api::*;
use gdnative::prelude::*;
use std::{cell::RefCell, path::Path};

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
        if let Some(dialog) = owner.get_node_as::<ConfirmationDialog>(&self.confirm) {
          *self.confirmation.borrow_mut() = Confirmation::Quit;
          unsafe {
            // Calling popup_centered directly from here causes an internal godot error.
            dialog.call_deferred(
              self.popup_centered.clone(),
              &[Variant::from_vector2(&Vector2::zero())],
            );
          }
          return;
        }
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

    // Disable the save button.
    self.enable_save(owner, false);

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
              if let Some(path) = Path::new(path.as_str()).file_name() {
                if let Some(path) = path.to_str() {
                  self.set_status_message(owner, &format!("Editing \"{}\"", path));
                }
              }
              return;
            }
            self.enable_gold(owner, None);
          }
        }
        self.disable_tree(owner, SkillTree::Producer);
      }
      self.disable_tree(owner, SkillTree::Adventurer);
    }

    // Set the status message.
    let path = path.to_utf8();
    if let Some(path) = Path::new(path.as_str()).file_name() {
      if let Some(path) = path.to_str() {
        self.set_status_message(owner, &format!("Unable to edit \"{}\"", path));
      }
    }
  }

  #[export]
  fn save_clicked(&self, owner: TRef<Node>) {
    if self.save(owner) {
      return;
    }

    let game_info = self.game_info.borrow();
    let game_info = some!(game_info.as_ref());
    let path = game_info.path().to_utf8();
    let path = some!(Path::new(path.as_str()).file_name());
    let path = some!(path.to_str());
    self.set_status_message(owner, &format!("Unable to save \"{}\"", path));
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
      let item = unsafe { item.assume_safe() };
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
    return true;
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
      unsafe { scene.assume_safe() }.quit(0);
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
          unsafe { edit.assume_safe() }.set_focus_mode(Control::FOCUS_ALL);
        }
      }
      None => {
        spin_box.set_value(0.0);
        spin_box.set_editable(false);
        spin_box.set_focus_mode(Control::FOCUS_NONE);
        if let Some(edit) = spin_box.get_line_edit() {
          unsafe { edit.assume_safe() }.set_focus_mode(Control::FOCUS_NONE);
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
          unsafe { edit.assume_safe() }.set_focus_mode(Control::FOCUS_ALL);
        }
      }
      None => {
        spin_box.set_value(0.0);
        spin_box.set_editable(false);
        spin_box.set_focus_mode(Control::FOCUS_NONE);
        if let Some(edit) = spin_box.get_line_edit() {
          unsafe { edit.assume_safe() }.set_focus_mode(Control::FOCUS_NONE);
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

    let parent = some!(tree.create_item(Object::null(), -1), false);
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

      let level = if let Some(val) = game_info.get_skill_exp(&id) {
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

      if let Some(item) = tree.create_item(parent, -1) {
        let item = unsafe { item.assume_safe() };
        // Skill name.
        item.set_custom_color(0, skill_color);
        item.set_text(0, GodotString::from(skill));

        // Experience multiplier.
        item.set_custom_color(1, info_color);
        item.set_text(1, GodotString::from(format!("{}x", mul)));

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

  fn collect_skills(&self, owner: TRef<Node>, tree: SkillTree, game_info: &mut GameInfo) -> bool {
    let tree = match tree {
      SkillTree::Adventurer => some!(owner.get_node_as::<Tree>(&self.adventurer), false),
      SkillTree::Producer => some!(owner.get_node_as::<Tree>(&self.producer), false),
    };

    let root = some!(tree.get_root(), false);
    let root = unsafe { root.assume_safe() };
    let mut node = root.get_children();
    loop {
      if let Some(item) = node {
        let item = unsafe { item.assume_safe() };

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
        node = item.get_next();
      } else {
        return true;
      }
    }
  }

  fn set_status_message(&self, owner: TRef<Node>, text: &str) {
    if let Some(label) = owner.get_node_as::<Label>(&self.status) {
      label.set_text(GodotString::from(text));
    }
  }
}
