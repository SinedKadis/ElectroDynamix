use bevy::prelude::Resource;

#[derive(Resource)]
pub struct Blocks{
    pub(crate) pos: Vec<(i32,i32)>,
}