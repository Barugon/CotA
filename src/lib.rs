#[macro_use]
mod util;

mod app;
mod experience;
mod constants;
mod offline;
mod portals;
mod stats;

use crate::app::*;
use crate::experience::*;
use crate::offline::*;
use crate::portals::*;
use crate::stats::*;
use gdnative::*;

fn init(handle: gdnative::init::InitHandle) {
  handle.add_class::<App>();
  handle.add_class::<Experience>();
  handle.add_class::<Offline>();
  handle.add_class::<Portals>();
  handle.add_class::<Stats>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
