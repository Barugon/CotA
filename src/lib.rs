#[macro_use]
mod util;

mod app;
mod stats;

use crate::app::*;
use crate::stats::*;
use gdnative::*;

fn init(handle: gdnative::init::InitHandle) {
  handle.add_class::<App>();
  handle.add_class::<Stats>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
