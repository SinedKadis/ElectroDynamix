use std::ops::Add;
use bevy::color::palettes::css::ORANGE_RED;
use bevy::color::palettes::tailwind::{CYAN_700, YELLOW_300, YELLOW_700};
use bevy::prelude::{Color, Gizmos, Isometry2d, Rot2, Vec2, Vec3};
use bevy_vector_shapes::prelude::{RectPainter, ShapePainter};
use crate::world::direction::Direction;

#[derive(Clone,Copy,PartialEq,Eq,Default)]
pub enum BlockState{
    #[default]
    Empty,
    Copper,
    Electricity([bool;4]),
    Diode(Direction,bool)
}

impl BlockState {

    pub fn get_color(&self) -> Color{
        match self {
            BlockState::Empty => Color::NONE,
            BlockState::Copper => Color::from(ORANGE_RED),
            BlockState::Electricity(_) => Color::from(CYAN_700),
            BlockState::Diode(_,act) => {
                if *act{
                    Color::from(YELLOW_300)
                }else{
                    Color::from(YELLOW_700)
                }
            },
        }
    }
    pub fn draw_preview(&self, world_pos: Vec2, gizmos: &mut Gizmos) {
        match self {
            BlockState::Empty => {
                gizmos.cross_2d(Isometry2d::new(world_pos.floor().add(Vec2::new(0.5,0.5)),Rot2::degrees(45.0)),
                                0.5,Color::WHITE);
            }
            BlockState::Copper => {
                gizmos.rect_2d(world_pos.floor().add(Vec2::new(0.5,0.5)), Vec2::ONE, self.get_color());
                gizmos.line_2d(world_pos.floor(), world_pos.floor() + Vec2::ONE, self.get_color());
            }
            BlockState::Electricity(directions) => {
                gizmos.rect_2d(world_pos.floor().add(Vec2::new(0.5,0.5)), Vec2::ONE, self.get_color());
                self.draw_arrows(world_pos, gizmos, directions, true, self.get_color());
            }
            BlockState::Diode(dir,_) => {
                gizmos.rect_2d(world_pos.floor().add(Vec2::new(0.5,0.5)), Vec2::ONE, self.get_color());
                self.draw_arrows(world_pos, gizmos, &dir.to_bool_array(), true, self.get_color());
            }
        }
    }

    pub fn draw(&self, world_pos: Vec2, gizmos: &mut Gizmos, painter: &mut ShapePainter, draw_arrows: bool ) {
        match self {
            BlockState::Empty => {}
            BlockState::Copper => {
                painter.color = self.get_color();
                painter.translate(Vec3::new(world_pos.x.floor() + 0.5, world_pos.y.floor() + 0.5, 1.0));
                painter.rect(Vec2::splat(1.0));
                painter.reset();
            }
            BlockState::Electricity(directions) => {
                painter.color = self.get_color();
                painter.translate(Vec3::new(world_pos.x.floor() + 0.5, world_pos.y.floor() + 0.5, 1.0));
                painter.rect(Vec2::splat(1.0));
                painter.reset();
                if draw_arrows {
                    self.draw_arrows(world_pos, gizmos, directions, false, Color::WHITE);
                }
            }
            BlockState::Diode(dir,_) => {
                painter.color = self.get_color();
                painter.translate(Vec3::new(world_pos.x.floor() + 0.5, world_pos.y.floor() + 0.5, 1.0));
                painter.rect(Vec2::splat(1.0));
                painter.reset();
                if draw_arrows {
                    self.draw_arrows(world_pos, gizmos, &dir.to_bool_array(), true, Color::BLACK);
                }
            }
        }
    }

    fn draw_arrows(&self, world_pos: Vec2, gizmos: &mut Gizmos, directions: &[bool; 4], centered: bool, color: Color) {
        for (i, dir) in directions.iter().enumerate() {
            if !dir { continue; }
            let direction = Direction::DIRECTIONS[i];
            let center = world_pos.floor().add(Vec2::new(0.5, 0.5));
            if centered {
                gizmos.arrow_2d(center.add(direction.opposite().to_vector() * 0.4), center.add(direction.to_vector() * 0.4), color);
            } else {
                gizmos.arrow_2d(center + (direction.to_vector() * 0.1), center + (direction.to_vector() * 0.9), color);
            }
        }
    }

    pub(crate) fn compute(&self, surround_states: [&BlockState;4]) -> BlockState {
        match self {
            BlockState::Empty => *self,
            BlockState::Copper => {
                let mut new_dir: [bool;4] = [false;4];
                for (i, surround_state) in surround_states.iter().enumerate() {
                    new_dir[(i+2)%4] = surround_state.is_activated_for(&Direction::DIRECTIONS[i]);
                }
                if new_dir.contains(&true) {
                    return  BlockState::Electricity(new_dir)
                }
                BlockState::Copper
            }
            BlockState::Electricity(directions) => {
                let mut new_dir: [bool;4] = [false;4];
                for (i, direction) in Direction::from_bool_array(*directions).iter().enumerate() {
                    if *direction == Direction::None { continue }
                    new_dir[i] = surround_states[(i+2)%4].is_activated_for(&direction.opposite());
                }
                if new_dir.contains(&true) {
                    return  BlockState::Electricity(new_dir)
                }
                BlockState::Copper
            }
            BlockState::Diode(dir,_act) => {
                let right = dir.rotate(true);
                if surround_states[right as usize].is_activated_for(&right) {
                    return BlockState::Diode(*dir,false)
                }
                let left = dir.rotate(false);
                if surround_states[left as usize].is_activated_for(&left) {
                    return BlockState::Diode(*dir,false)
                }
                BlockState::Diode(*dir,surround_states[((*dir as usize)+2)%4].is_activated_for(&dir.opposite()))
            }
        }
    }

    ///Using the opposite direction - when the request comes from right, the direction is left
    fn is_activated_for(&self, request_from: &Direction) -> bool {
        match self {
            BlockState::Electricity(dir) => {
                if *request_from != Direction::None && dir[*request_from as usize] {
                    return false;
                }
                dir.contains(&true)
            },
            BlockState::Diode(dir, act) => *act && *dir == request_from.opposite(),
            _ => false,
        }
    }

    pub fn rotate(&self,clockwise: bool) -> BlockState {
        match self {
            BlockState::Empty => {*self}
            BlockState::Copper => {*self}
            BlockState::Electricity(dir) => {
                let x: Vec<Direction> = Direction::from_bool_array(*dir).iter()
                    .map(|dir| Direction::rotate(dir,clockwise))
                    .collect();
                if let Some(new_dir) = x.as_array() {
                    return BlockState::Electricity(Direction::bool_array_from_array(*new_dir));
                }
                *self
            },
            BlockState::Diode(dir,a) => {
                BlockState::Diode(Direction::rotate(dir,clockwise), *a)
            }
        }
    }
}