use crate::world::blockstate::BlockState;
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct DrewBlocks {
    pub(crate) block_data: Vec<(i64, i64, BlockState)>,
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

    pub fn add_or_replace_block(&mut self, state: BlockState, x: i64, y: i64) {
        if let Some(existing_block) = self
            .block_data
            .iter_mut()
            .find(|(bx, by, _)| *bx == x && *by == y)
        {
            existing_block.2 = state;
        } else {
            self.block_data.push((x, y, state));
        }
    }
}

#[derive(Serialize, Deserialize)]
struct StateGroup {
    state: BlockState,
    positions: Vec<[i64; 2]>,
}

#[derive(Serialize, Deserialize)]
pub struct DrewBlocksFile {
    block_data: Vec<StateGroup>,
}

impl From<&DrewBlocks> for DrewBlocksFile {
    fn from(blocks: &DrewBlocks) -> Self {
        Self::blocks_to_state_group(&blocks.block_data) 
    }
}

impl DrewBlocksFile {
    pub fn blocks_to_state_group(blocks: &Vec<(i64, i64, BlockState)>) -> Self {
        let mut groups: Vec<StateGroup> = Vec::new();

        for (x, y, state) in blocks {
            if *state == BlockState::Empty {
                continue;
            }
            match groups.iter_mut().find(|g| g.state == *state) {
                Some(group) => group.positions.push([*x, *y]),
                None => groups.push(StateGroup { state: *state, positions: vec![[*x, *y]] }),
            }
        }
        DrewBlocksFile { block_data:groups }
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