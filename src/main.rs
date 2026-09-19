pub mod control;
mod window;
mod world;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            control::ControlPlugin,
            window::WindowPlugin,
            world::WorldPlugin,
        ))
        .run();
}
