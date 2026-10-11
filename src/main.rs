#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod plugins;
mod world;

use world::world::WorldPlugin;
use bevy::prelude::*;
use plugins::{config, control, window};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ElectroDinamix".into(),
                name: Some("ElectroDinamix".into()),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            control::ControlPlugin,
            window::UiPlugin,
            WorldPlugin,
            config::ConfigPlugin,
        ))
        .run();
}
