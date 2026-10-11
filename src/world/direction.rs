use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone,Copy,PartialEq,Eq, Serialize, Deserialize)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
    None
}

impl Direction {
    pub const DIRECTIONS: [Direction;4] = [Direction::Left, Direction::Up, Direction::Right,  Direction::Down];
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::None => Direction::None,
        }
    }

    pub fn to_tuple_int(&self) -> (i32, i32) {
        match self {
            Direction::Left => (-1,0),
            Direction::Right => (1,0),
            Direction::Up => (0,1),
            Direction::Down => (0,-1),
            Direction::None => (0,0),
        }
    }

    pub fn to_vector(&self) -> Vec2 {
        match self {
            Direction::Left => Vec2::new(-1.0,0.0),
            Direction::Right => Vec2::new(1.0,0.0),
            Direction::Up => Vec2::new(0.0,1.0),
            Direction::Down => Vec2::new(0.0,-1.0),
            Direction::None => Vec2::new(0.0,0.0),
        }
    }

    pub fn to_bool_array(&self) -> [bool;4]{
        match self {
            Direction::Left => [true,false,false,false],
            Direction::Right => [false,false,true,false],
            Direction::Up => [false,true,false,false],
            Direction::Down => [false,false,false,true],
            Direction::None => [false,false,false,false],
        }
    }

    pub fn bool_array_from_array(arr: [Direction;4]) -> [bool;4] {
        let mut bool_array = [false;4];
        if arr.contains(&Direction::Left) { bool_array[0] = true };
        if arr.contains(&Direction::Up) { bool_array[1] = true };
        if arr.contains(&Direction::Right) { bool_array[2] = true };
        if arr.contains(&Direction::Down) { bool_array[3] = true };
        bool_array
    }

    pub fn from_bool_array(arr: [bool;4]) -> [Direction;4]{
        let mut directions = [Direction::None;4];
        if arr[0] { directions[0] = Direction::Left } else { directions[0] = Direction::None };
        if arr[1] { directions[1] = Direction::Up } else { directions[1] = Direction::None };
        if arr[2] { directions[2] = Direction::Right } else { directions[2] = Direction::None };
        if arr[3] { directions[3] = Direction::Down } else { directions[3] = Direction::None };
        directions
    }

    pub fn rotate(&self,clockwise: bool) -> Direction{
        if clockwise {
            match self {
                Direction::Left => Direction::Up,
                Direction::Right => Direction::Down,
                Direction::Up => Direction::Right,
                Direction::Down => Direction::Left,
                Direction::None => Direction::None,
            }
        } else {
            match self {
                Direction::Left => Direction::Down,
                Direction::Right => Direction::Up,
                Direction::Up => Direction::Left,
                Direction::Down => Direction::Right,
                Direction::None => Direction::None,
            }
        }

    }
}