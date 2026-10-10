use crate::world::blockstate::BlockState;
use crate::world::direction::Direction;
use bevy::camera::Camera;
use bevy::input::ButtonInput;
use bevy::math::{USizeVec2, Vec2};
use bevy::prelude::MouseButton::Left;
use bevy::prelude::{Gizmos, GlobalTransform, MouseButton, Res, Single, Window};
use bevy_vector_shapes::painter::ShapePainter;

#[derive(Clone,PartialEq,Eq)]
pub(crate) struct GameMap {
    width: usize,
    height: usize,
    min_x: i32,
    min_y: i32,
    cells: Vec<BlockState>,
}

impl GameMap {
    pub(crate) fn new(width: usize, height: usize, min_x: i32, min_y: i32) -> Self {
        GameMap {
            width,
            height,
            min_x,
            min_y,
            cells: vec![BlockState::default(); width * height],
        }
    }

    fn index(&self, xy: USizeVec2) -> usize {
        xy.y * self.width + xy.x
    }

    pub(crate) fn get(&self, x: i32, y: i32) -> Option<&BlockState> {
        let xy = self.to_game_pos(x, y)?;
        self.cells.get(self.index(xy))
    }

    pub fn to_game_pos(&self, x: i32, y: i32) -> Option<USizeVec2> {
        if x < self.min_x
            || x >= self.min_x + self.width as i32
            || y < self.min_y
            || y >= self.min_y + self.height as i32 {
            return None;
        }
        Some(USizeVec2::new((x - self.min_x) as usize, (y - self.min_y) as usize))
    }

    pub(crate) fn set(&mut self, x: i32, y: i32, state: BlockState) {
        if let Some(xy) = self.to_game_pos(x, y) {
            let idx = self.index(xy);
            self.cells[idx] = state;
        }
    }

    pub(crate) fn draw(&self, painter: &mut ShapePainter, gizmos: &mut Gizmos, draw_arrows: bool) {
        for gy in 0..self.height {
            for gx in 0..self.width {
                let state = &self.cells[gy * self.width + gx];

                let x = self.min_x + gx as i32;
                let y = self.min_y + gy as i32;

                state.draw(Vec2::new(x as f32, y as f32), gizmos ,painter, draw_arrows);
            }
        }
    }

    fn get_cell(&self, xy: (usize,  usize),offset: (i32, i32)) -> Option<&BlockState> {
        let x = xy.0 as i32 + offset.0;
        let y = xy.1 as i32 + offset.1;

        if x < 0 || y < 0 {
            return None;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.width || y >= self.height {
            return None;
        }
        self.cells.get(y * self.width + x)
    }

    pub(crate) fn update(& self,
                         mouse_button_input: Res<ButtonInput<MouseButton>>,
                         window: Single<&Window>,
                         camera_query: Single<(&Camera, &GlobalTransform)>) -> GameMap {
        let mut new_game_map = self.clone();
        let interact_pos: Option<USizeVec2>;
        if mouse_button_input.pressed(Left) {
            let (camera, camera_transform) = camera_query.into_inner();
            if let Some(cursor_position) = window.cursor_position()  {
                if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
                   interact_pos = self.to_game_pos(world_pos.x.floor() as i32, world_pos.y.floor() as i32);
                } else { interact_pos = None; };
            } else { interact_pos = None; };
        } else { interact_pos = None; }
        for y in 0..self.height {
            for x in 0..self.width {
                let interact: bool;
                if let Some(inter_pos) = interact_pos {
                    interact = inter_pos.x == x && inter_pos.y == y;
                } else { interact = false; }
                let state = &self.cells[y * self.width + x];
                new_game_map.cells[y * self.width + x] = state.compute(Direction::DIRECTIONS.iter()
                    .map(|d| self.get_cell((x,y),d.to_tuple_int())
                        .unwrap_or(&BlockState::Empty))
                    .collect::<Vec<&BlockState>>()
                    .try_into()
                    .unwrap_or([&BlockState::Empty;4]),interact
                );
            }
        }

        new_game_map
    }
}