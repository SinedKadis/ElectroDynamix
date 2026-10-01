use crate::plugins::config::Config;
use bevy::color::palettes::css::ORANGE_RED;
use bevy::color::palettes::tailwind::CYAN_700;
use bevy::prelude::*;
use bevy::reflect::array::Array;
use bevy_vector_shapes::Shape2dPlugin;
use bevy_vector_shapes::painter::ShapePainter;
use bevy_vector_shapes::shapes::RectPainter;
use std::ops::Add;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, draw_preview.after(TransformSystems::Propagate));
        app.add_plugins(Shape2dPlugin::default());
        app.insert_resource(Blocks{ block_data: Vec::new()});
        app.insert_resource(GameState{state: GameStates::Drawing});
        app.insert_resource(UpdateTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
        app.add_systems(Update, draw_blocks);
        app.add_systems(FixedUpdate, update_blocks);
    }
}


fn draw_preview(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut gizmos: Gizmos,
    config: Res<Config>,
    state: Res<SelectedState>
) {
    if !config.draw_grid { return; }
    let (camera, camera_transform) = *camera_query;

    if let Some(cursor_position) = window.cursor_position()
        && let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position)
    {
        for pos in get_positions_in_range(world_pos.floor().as_ivec2(), state.size-1) {
            state.state.draw_preview(pos.as_vec2(), &mut gizmos)
        }
    }
}
fn draw_blocks(mut painter: ShapePainter,
               blocks: Res<Blocks>,
               mut gizmos: Gizmos,
               config : Res<Config>,
               game_state: Res<GameState>) {
    match &game_state.state {
        GameStates::Drawing => {
            for block_data in &blocks.block_data {
                let state = block_data.2;
                state.draw(Vec2::new(block_data.0 as f32, block_data.1 as f32), &mut gizmos, &mut painter,config.draw_arrows);
            }
        }

        GameStates::Compiling => {}
        GameStates::Executing(game_map) => {
            game_map.draw(&mut painter,&mut gizmos, config.draw_arrows);
        }
        GameStates::Paused(game_map) => {
            game_map.draw(&mut painter,&mut gizmos, config.draw_arrows);
        }
    }


}

#[derive(Resource)]
pub struct Blocks{
    pub(crate) block_data: Vec<(i32, i32, BlockState)>,
}

#[derive(Clone,Copy,PartialEq,Eq,Default)]
pub enum BlockState{
    #[default]
    Empty,
    Copper,
    Electricity([bool;4])
}

impl BlockState {

    pub fn get_color(&self) -> Color{
        match self {
            BlockState::Empty => Color::NONE,
            BlockState::Copper => Color::from(ORANGE_RED),
            BlockState::Electricity(_) => Color::from(CYAN_700),
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
                self.draw_arrows(world_pos, gizmos, directions,true);
            }
        }
    }

    pub(crate) fn draw(&self, world_pos: Vec2, gizmos: &mut Gizmos, painter: &mut ShapePainter, draw_arrows: bool ) {
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
                    self.draw_arrows(world_pos, gizmos, directions,false);
                }
            }
        }
    }

    fn draw_arrows(&self, world_pos: Vec2, gizmos: &mut Gizmos, directions: &[bool; 4], preview: bool) {
        for (i, dir) in directions.iter().enumerate() {
            if !dir.try_downcast_ref::<bool>().unwrap() { continue; }
            let direction = Direction::DIRECTIONS[i];
            let center = world_pos.floor().add(Vec2::new(0.5, 0.5));
            if preview {
                gizmos.arrow_2d(center.add(direction.opposite().to_vector() * 0.4), center.add(direction.to_vector() * 0.4), self.get_color());
            } else {
                gizmos.arrow_2d(center + (direction.to_vector() * 0.1), center + (direction.to_vector() * 0.9), Color::WHITE);
            }
        }
    }

    fn compute(&self, surround_states: [&BlockState;4]) -> BlockState {
        match self {
            BlockState::Copper => {
                let mut new_dir: [bool;4] = [false;4];
                for (i, surround_state) in surround_states.iter().enumerate() {
                    if let BlockState::Electricity(elect_dirs) = surround_state {
                        if !elect_dirs[i] {
                            new_dir[(i+2)%4] = true;
                        }
                    }
                }
                if new_dir.contains(&true) {
                    return  BlockState::Electricity(new_dir)
                }
                BlockState::Copper
            }
            BlockState::Electricity(directions) => {
                let mut new_dir: [bool;4] = [false;4];
                for (i, _direction) in directions.iter().enumerate() {
                    if let BlockState::Electricity(elect_dirs) = surround_states[(i+2)%4] {
                        if !elect_dirs[(i+2)%4] {
                            new_dir[i] = true;
                        }
                    }
                }
                if new_dir.contains(&true) {
                    return  BlockState::Electricity(new_dir)
                }
                BlockState::Copper
            }
            _ => {*self}
        }
    }
}


#[derive(Clone,Copy,PartialEq,Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}


impl Direction {
    pub const DIRECTIONS: [Direction;4] = [Direction::Left, Direction::Up, Direction::Right,  Direction::Down];
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }

    pub fn to_tuple_int(&self) -> (i32, i32) {
        match self {
            Direction::Left => (-1,0),
            Direction::Right => (1,0),
            Direction::Up => (0,1),
            Direction::Down => (0,-1),
        }
    }

    pub fn to_vector(&self) -> Vec2 {
        match self {
            Direction::Left => Vec2::new(-1.0,0.0),
            Direction::Right => Vec2::new(1.0,0.0),
            Direction::Up => Vec2::new(0.0,1.0),
            Direction::Down => Vec2::new(0.0,-1.0),
        }
    }
}

#[derive(Resource)]
pub struct SelectedState{
    pub(crate) state: BlockState,
    pub(crate) size: i32,
}

#[derive(Resource)]
pub struct GameState{
    pub(crate) state: GameStates,
}

#[derive(Clone,PartialEq,Eq)]
pub enum GameStates{
    Drawing,
    Compiling,
    Executing(GameMap),
    Paused(GameMap),
}

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

    fn index(&self, x: i32, y: i32) -> usize {
        let gx = (x - self.min_x) as usize;
        let gy = (y - self.min_y) as usize;
        gy * self.width + gx
    }

    pub(crate) fn set(&mut self, x: i32, y: i32, state: BlockState) {
        let idx = self.index(x, y);
        self.cells[idx] = state;
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
    
    pub(crate) fn update(& self) -> GameMap {
        let mut new_game_map = self.clone();
        for y in 0..self.height {
            for x in 0..self.width {
                let state = &self.cells[y * self.width + x];
                new_game_map.cells[y * self.width + x] = state.compute(Direction::DIRECTIONS.iter()
                    .map(|d| self.get_cell((x,y),d.to_tuple_int())
                        .unwrap_or(&BlockState::Empty))
                    .collect::<Vec<&BlockState>>()
                    .try_into()
                    .unwrap_or([&BlockState::Empty;4])
                );
            }
        }

        new_game_map
    }
}

#[derive(Resource)]
pub(crate) struct UpdateTimer(pub(crate) Timer);

fn update_blocks(mut game_state: ResMut<GameState>,
    mut timer: ResMut<UpdateTimer>,
    time: Res<Time>) {
    if timer.0.tick(time.delta()).just_finished() {
        let new_game_map : GameMap = match &game_state.state {
            GameStates::Drawing => {return;}
            GameStates::Compiling => {return;}
            GameStates::Executing(game_map) => {game_map.update()}
            GameStates::Paused(_) => {return;}
        };
        game_state.state = GameStates::Executing(new_game_map);
    }
}

pub fn get_positions_in_range(center: IVec2, max_distance: i32) -> Vec<IVec2> {
    let mut points = Vec::new();

    for x in -max_distance..=max_distance {
        for y in -max_distance..=max_distance {
            let point = center + IVec2::new(x, y);

            if center.distance_squared(point) <= (max_distance*max_distance) {
                points.push(point);
            }
        }
    }
    points
}