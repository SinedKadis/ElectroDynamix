
mod plugins;
mod world;

use world::world::WorldPlugin;
use bevy::prelude::*;
use plugins::{config, control, window};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            control::ControlPlugin,
            window::WindowPlugin,
            WorldPlugin,
            config::ConfigPlugin,
        ))
        .run();
}
