use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use crate::world::blockstate::BlockState;

#[derive(Resource)]
pub struct DrewBlocks {
    pub(crate) block_data: Vec<(i32, i32, BlockState)>,
}

impl DrewBlocks {
    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = DrewBlocksFile::from(self);
        std::fs::write(path, serde_json::to_string_pretty(&file)?)?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let text = std::fs::read_to_string(path)?;
        let file: DrewBlocksFile = serde_json::from_str(&text)?;
        Ok(file.into())
    }
}

#[derive(Serialize, Deserialize)]
struct StateGroup {
    state: BlockState,
    positions: Vec<[i32; 2]>,
}

#[derive(Serialize, Deserialize)]
struct DrewBlocksFile {
    block_data: Vec<StateGroup>,
}

impl From<&DrewBlocks> for DrewBlocksFile {
    fn from(blocks: &DrewBlocks) -> Self {
        let mut groups: Vec<StateGroup> = Vec::new();
        for &(x, y, state) in &blocks.block_data {
            if state == BlockState::Empty {
                continue;
            }
            match groups.iter_mut().find(|g| g.state == state) {
                Some(group) => group.positions.push([x, y]),
                None => groups.push(StateGroup { state, positions: vec![[x, y]] }),
            }
        }
        DrewBlocksFile { block_data: groups }
    }
}

impl From<DrewBlocksFile> for DrewBlocks {
    fn from(file: DrewBlocksFile) -> Self {
        let block_data = file
            .block_data
            .into_iter()
            .flat_map(|g| {
                let state = g.state;
                g.positions.into_iter().map(move |[x, y]| (x, y, state))
            })
            .collect();
        DrewBlocks { block_data }
    }
}