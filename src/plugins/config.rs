use bevy::app::{App, Plugin};
use bevy::prelude::Resource;

pub struct ConfigPlugin;
impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Config
        {
            draw_grid: true, 
            draw_arrows: true
        });
    }
}
#[derive(Resource)]
pub struct Config{
    pub draw_grid : bool,
    pub draw_arrows : bool,
}