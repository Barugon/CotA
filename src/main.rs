#![windows_subsystem = "windows"]

#[macro_use]
mod util;

mod experience_page;
mod portals_page;
mod ui;

use gio::prelude::{ApplicationExt, ApplicationExtManual};
use std::env;

fn main() {
  let app = gtk::Application::new(Some("org.barugon.cota"), gio::ApplicationFlags::NON_UNIQUE)
    .expect("Initialization failed!");

  app.connect_startup(ui::layout);
  app.connect_activate(|_| {});
  app.run(&env::args().collect::<Vec<_>>());
}
