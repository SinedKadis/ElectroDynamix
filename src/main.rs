mod world;
pub mod mouse_control;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((world::WorldPlugin, mouse_control::ControlPlugin))
        // .add_plugins(DefaultPlugins.set(WindowPlugin {
        //     primary_window: Some(Window {
        //         title: "ElectroDynamix".into(),
        //         resolution: (800, 600).into(),
        //         resizable: true,
        //         ..default()
        //     }),
        //     ..default()
        // }))
        .run();
}
