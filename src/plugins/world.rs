use crate::plugins::config::Config;
use bevy::color::palettes::basic::WHITE;
use bevy::color::palettes::css::ORANGE_RED;
use bevy::color::palettes::tailwind::CYAN_700;
use bevy::prelude::*;
use bevy_vector_shapes::Shape2dPlugin;
use bevy_vector_shapes::painter::ShapePainter;
use bevy_vector_shapes::shapes::RectPainter;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, draw_grid.after(TransformSystems::Propagate));
        app.add_plugins(Shape2dPlugin::default());
        app.insert_resource(Blocks{ block_data: Vec::new()});
        app.insert_resource(GameState{state: GameStates::Drawing});
        app.insert_resource(UpdateTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
        app.add_systems(Update, draw_blocks);
        app.add_systems(FixedUpdate, update_blocks);
    }
}

const GRID_COLOR: Srgba = WHITE;

fn draw_grid(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut gizmos: Gizmos,
    config: Res<Config>
) {
    if !config.draw_grid { return; }
    let (camera, camera_transform) = *camera_query;

    if let Some(cursor_position) = window.cursor_position()
        && let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position)
    {
        gizmos.grid_2d(world_pos.round(), UVec2::new(10, 10), Vec2::new(1., 1.), GRID_COLOR);
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
                let visual_y = block_data.1 as f32 + 0.5;
                let visual_x = block_data.0 as f32 + 0.5;
                painter.color = match block_data.2 {
                    BlockState::Copper => {Color::from(ORANGE_RED)}
                    BlockState::Electricity(dir) => {
                        if config.draw_arrows {
                            for direction in dir {
                                let dir_vec = direction.opposite().to_vector();
                                gizmos.arrow_2d(
                                    Vec2::new(dir_vec.0 as f32 + visual_x, dir_vec.1 as f32 + visual_y),
                                    Vec2::new(visual_x, visual_y),
                                    WHITE
                                );
                            }
                        }
                        Color::from(CYAN_700)
                    }
                    BlockState::Empty => {continue}
                };
                painter.translate(Vec3::new(visual_x, visual_y, 1.0));

                painter.rect(Vec2::splat(1.0));

                painter.reset();
            }
        }
        GameStates::Compiling => {}
        GameStates::Executing(game_map) => {
            game_map.draw(&mut painter,&mut gizmos, &config.draw_arrows);
        }
        GameStates::Paused(game_map) => {
            game_map.draw(&mut painter,&mut gizmos, &config.draw_arrows);
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
    Electricity([Direction;4])
}

#[derive(Clone,Copy,PartialEq,Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    None
}

trait ComputableState{
    ///surround dirs: left, right, up, down
    fn compute(&self, surround_states: [&BlockState;4]) -> BlockState;
}

impl ComputableState for BlockState{
    fn compute(&self, surround_states: [&BlockState;4]) -> BlockState {
        match self {
            BlockState::Copper => {
                let mut new_dir: [Direction;4] = [Direction::None;4];
                for (i, surround_state) in surround_states.iter().enumerate() {
                    if let BlockState::Electricity(elect_dirs) = surround_state {
                        let dir = Direction::DIRECTIONS[i];
                        if !elect_dirs.contains(&dir) {
                            new_dir[i] = dir.opposite();
                        }
                    }
                }
                if new_dir.iter().any(|dir| dir != &Direction::None) {
                    return  BlockState::Electricity(new_dir)
                }
                BlockState::Copper
            }
            BlockState::Electricity(directions) => {
                let mut new_dir: [Direction;4] = [Direction::None;4];
                for (i, direction) in directions.iter().enumerate() {
                    let idx = Direction::DIRECTIONS.iter().position(|d| *d == direction.opposite());
                    if idx.is_none() { continue }
                    if let BlockState::Electricity(elect_dirs) = surround_states[idx.unwrap()] {
                        if !elect_dirs.contains(&direction.opposite()) {
                            new_dir[i] = *direction;
                        }
                    }
                }
                if new_dir.iter().any(|dir| dir != &Direction::None) {
                    return  BlockState::Electricity(new_dir)
                }
                BlockState::Copper
            }
            _ => {*self}
        }
    }
}

impl Direction {
    pub const DIRECTIONS: [Direction;4] = [Direction::Left, Direction::Right, Direction::Up, Direction::Down];
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            _ => {*self}
        }
    }

    pub fn to_vector(&self) -> (i32,i32) {
        match self {
            Direction::Left => (-1,0),
            Direction::Right => (1,0),
            Direction::Up => (0,1),
            Direction::Down => (0,-1),
            _ => (0,0)
        }
    }
}

#[derive(Resource)]
pub struct SelectedState{
    pub(crate) state: BlockState,
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

    pub(crate) fn draw(&self, painter: &mut ShapePainter, gizmos: &mut Gizmos, draw_arrows: &bool) {
        for gy in 0..self.height {
            for gx in 0..self.width {
                let state = &self.cells[gy * self.width + gx];

                let x = self.min_x + gx as i32;
                let y = self.min_y + gy as i32;
                let visual_x = x as f32 + 0.5;
                let visual_y = y as f32 + 0.5;

                painter.color = match state {
                    BlockState::Copper => Color::from(ORANGE_RED),
                    BlockState::Electricity(dir) => {
                        if *draw_arrows {
                            for direction in dir {
                                let dir_vec = direction.opposite().to_vector();
                                gizmos.arrow_2d(
                                    Vec2::new(dir_vec.0 as f32 + visual_x, dir_vec.1 as f32 + visual_y),
                                    Vec2::new(visual_x, visual_y),
                                    WHITE
                                );
                            }
                        }
                        Color::from(CYAN_700)
                    }
                    _ => continue,
                };

                painter.translate(Vec3::new(visual_x, visual_y, 1.0));
                painter.rect(Vec2::splat(1.0));
                painter.reset();
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
                    .map(|d| self.get_cell((x,y),d.to_vector())
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