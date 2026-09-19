use bevy::prelude::Resource;

#[derive(Resource)]
pub struct Blocks{
    pub(crate) x: Vec<i32>,
    pub(crate) y: Vec<i32>,
}