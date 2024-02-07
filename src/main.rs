mod cell_system;
mod gui;

use bevy::prelude::*;
use cell_system::CellSystem;
use gui::GuiSystem;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Simulation".into(),
                fit_canvas_to_parent: true,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(CellSystem)
        .add_plugins(GuiSystem)
        .run();
}