#[macro_use]
mod util;

mod app;
mod constants;
mod experience;
mod offline;
mod portals;
mod stats;
mod thread_pool;

use crate::app::*;
use crate::experience::*;
use crate::offline::*;
use crate::portals::*;
use crate::stats::*;
use gdnative::prelude::*;

fn init(handle: InitHandle) {
  handle.add_class::<App>();
  handle.add_class::<Experience>();
  handle.add_class::<Offline>();
  handle.add_class::<Portals>();
  handle.add_class::<Stats>();
}

godot_init!(init);
