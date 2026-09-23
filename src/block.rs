use bevy::prelude::Resource;

#[derive(Resource)]
pub struct Blocks{
    pub(crate) block_data: Vec<(i32, i32, BlockState)>,
}

#[derive(Clone,Copy,PartialEq,Eq)]
pub enum BlockState{
    Copper,
    Electricity(Direction),
    Source
}
#[derive(Clone,Copy,PartialEq,Eq)]
pub enum Direction{
    Up,Down,Left,Right
}

#[derive(Resource)]
pub struct SelectedState{
    pub(crate) state: BlockState,
}


