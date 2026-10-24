use crate::world::blockstate::BlockState;
use crate::world::direction::Direction;
use bevy::math::{I64Vec2, U64Vec2, Vec2};
use bevy::prelude::Gizmos;
use bevy_vector_shapes::painter::ShapePainter;
use rayon::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct GameMap {
    width: u64,
    height: u64,
    min_x: i64,
    min_y: i64,
    cells: Vec<BlockState>,
    held: Option<u64>
}

impl GameMap {
    pub(crate) fn new(width: u64, height: u64, min_x: i64, min_y: i64) -> Self {
        GameMap {
            width,
            height,
            min_x,
            min_y,
            cells: vec![BlockState::default(); (width * height) as usize],
            held: None
        }
    }

    fn index(&self, xy: U64Vec2) -> u64 {
        xy.y * self.width + xy.x
    }

    pub(crate) fn get(&self, x: i64, y: i64) -> Option<&BlockState> {
        let xy = self.to_game_pos(x, y)?;
        self.cells.get(self.index(xy) as usize)
    }

    pub fn to_game_pos(&self, x: i64, y: i64) -> Option<U64Vec2> {
        if x < self.min_x
            || x >= self.min_x + self.width as i64
            || y < self.min_y
            || y >= self.min_y + self.height as i64 {
            return None;
        }
        Some(U64Vec2::new((x - self.min_x) as u64, (y - self.min_y) as u64))
    }

    pub(crate) fn set(&mut self, x: i64, y: i64, state: BlockState) {
        if let Some(xy) = self.to_game_pos(x, y) {
            let idx = self.index(xy);
            self.cells[idx as usize] = state;
        }
    }

    pub(crate) fn draw(&self,min:I64Vec2,max:I64Vec2, painter: &mut ShapePainter, gizmos: &mut Gizmos, draw_arrows: bool) {

        let x0 = (min.x - self.min_x).clamp(0, self.width as i64) as u64;
        let x1 = (max.x - self.min_x + 1).clamp(0, self.width as i64) as u64;
        let y0 = (min.y - self.min_y).clamp(0, self.height as i64) as u64;
        let y1 = (max.y - self.min_y + 1).clamp(0, self.height as i64) as u64;

        for gy in y0..y1 { for gx in x0..x1 {
            let state = &self.cells[(gy * self.width + gx) as usize];

                let x = self.min_x + gx as i64;
                let y = self.min_y + gy as i64;

                state.draw(Vec2::new(x as f32, y as f32), gizmos ,painter, draw_arrows);
            }
        }
    }

    fn get_cell(&self, xy: (u64,  u64),offset: (i8, i8)) -> Option<&BlockState> {
        let x = xy.0 as i64 + offset.0  as i64;
        let y = xy.1 as i64 + offset.1  as i64;

        if x < 0 || y < 0 {
            return None;
        }
        let (x, y) = (x as u64, y as u64);
        if x >= self.width || y >= self.height {
            return None;
        }
        self.cells.get((y * self.width + x) as usize)
    }

    pub(crate) fn set_held(&mut self, pos: Option<U64Vec2>) {
        self.held = pos
            .map(|p| self.index(p))
            .filter(|&i| matches!(self.cells.get(i as usize), Some(BlockState::Button(_))));
        if let Some(i) = self.held {
            self.cells[i as usize] = BlockState::Button(true);
        }
    }


    const PAR_THRESHOLD: usize = 16_384;

    fn step_row(&self, y: u64, row: &mut [BlockState]) {
        for (x, slot) in row.iter_mut().enumerate() {
            let state = &self.cells[(y * self.width ) as usize + x];
            if *state == BlockState::Empty {
                continue;
            }
            let neighbors: [&BlockState; 4] = std::array::from_fn(|k| {
                self.get_cell((x as u64, y), Direction::DIRECTIONS[k].to_tuple_int())
                    .unwrap_or(&BlockState::Empty)
            });
            let i = y * self.width + x as u64;
            let new_state ;

            if let BlockState::Button(_) = state {
                new_state = BlockState::Button(self.held == Some(i));
            } else {
                new_state = state.compute(neighbors);
            }



            debug_assert!(new_state != BlockState::Empty, "клетка стала Empty");
            *slot = new_state;
        }
    }

    fn step_into(&self, out: &mut [BlockState]) {
        let width = self.width;
        if width == 0 || self.height == 0 {
            return;
        }
        if self.cells.len() < Self::PAR_THRESHOLD {
            out.chunks_mut(width as usize)
                .enumerate()
                .for_each(|(y, row)| self.step_row(y as u64, row));
        } else {
            out.par_chunks_mut(width as usize)
                .enumerate()
                .for_each(|(y, row)| self.step_row(y as u64, row));
        }
    }

    pub(crate) fn step_n(&mut self, n: u32) {
        let mut next = self.cells.clone();
        for _ in 0..n {
            self.step_into(&mut next);
            std::mem::swap(&mut self.cells, &mut next);
        }
    }
}