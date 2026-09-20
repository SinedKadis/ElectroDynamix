
mod block;
mod plugins;

use bevy::prelude::*;
use plugins::{config, control, window, world};


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            control::ControlPlugin,
            window::WindowPlugin,
            world::WorldPlugin,
            config::ConfigPlugin,
        ))
        .run();
}
