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
                            gizmos.arrow_2d(match dir {
                                Direction::Up => {Vec2::new(visual_x, visual_y-0.4)}
                                Direction::Down => {Vec2::new(visual_x,visual_y+0.4)}
                                Direction::Left => {Vec2::new(visual_x+0.4, visual_y)}
                                Direction::Right => {Vec2::new(visual_x-0.4, visual_y)}
                            }, match dir {
                                Direction::Up => {Vec2::new(visual_x, visual_y + 0.4)}
                                Direction::Down => {Vec2::new(visual_x, visual_y - 0.4)}
                                Direction::Left => {Vec2::new(visual_x - 0.4, visual_y)}
                                Direction::Right => {Vec2::new(visual_x + 0.4, visual_y)}
                            }, WHITE);
                        }
                        Color::from(CYAN_700)
                    }
                    _ => {continue}
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
    Electricity(Direction),
}

#[derive(Clone,Copy,PartialEq,Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
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

    fn get(&self, x: i32, y: i32) -> &BlockState {
        &self.cells[self.index(x, y)]
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
                            gizmos.arrow_2d(
                                match dir {
                                    Direction::Up => Vec2::new(visual_x, visual_y - 0.4),
                                    Direction::Down => Vec2::new(visual_x, visual_y + 0.4),
                                    Direction::Left => Vec2::new(visual_x + 0.4, visual_y),
                                    Direction::Right => Vec2::new(visual_x - 0.4, visual_y),
                                },
                                match dir {
                                    Direction::Up => Vec2::new(visual_x, visual_y + 0.4),
                                    Direction::Down => Vec2::new(visual_x, visual_y - 0.4),
                                    Direction::Left => Vec2::new(visual_x - 0.4, visual_y),
                                    Direction::Right => Vec2::new(visual_x + 0.4, visual_y),
                                },
                                WHITE,
                            );
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

    fn get_cell(&self, x: i32, y: i32) -> Option<&BlockState> {
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
                match state {
                    BlockState::Empty => {
                        continue;
                    }
                    BlockState::Copper => {
                        if let Some(left_cell) = self.get_cell(x as i32 - 1, y as i32) {
                            if left_cell != &BlockState::Electricity(Direction::Left) && matches!(left_cell, BlockState::Electricity(_)) {
                                new_game_map.cells[y * self.width + x] = BlockState::Electricity(Direction::Right);
                                continue
                            }
                        }

                        if let Some(right_cell) = self.get_cell(x as i32 + 1, y as i32) {
                            if right_cell != &BlockState::Electricity(Direction::Right) && matches!(right_cell, BlockState::Electricity(_)) {
                                new_game_map.cells[y * self.width + x] = BlockState::Electricity(Direction::Left);
                                continue
                            }
                        }

                        if let Some(up_cell) = self.get_cell(x as i32, y as i32 + 1) {
                            if up_cell != &BlockState::Electricity(Direction::Up) && matches!(up_cell, BlockState::Electricity(_)) {
                                new_game_map.cells[y * self.width + x] = BlockState::Electricity(Direction::Down);
                                continue
                            }
                        }

                        if let Some(down_cell) = self.get_cell(x as i32, y as i32 - 1) {
                            if down_cell != &BlockState::Electricity(Direction::Down) && matches!(down_cell, BlockState::Electricity(_)) {
                                new_game_map.cells[y * self.width + x] = BlockState::Electricity(Direction::Up);
                                continue
                            }
                        }
                    }
                    BlockState::Electricity(direction) => {
                        match direction {
                            Direction::Up => {
                                if let Some(down_cell) = self.get_cell(x as i32, y as i32 - 1) {
                                    if matches!(down_cell, BlockState::Electricity(Direction::Down))
                                        || !matches!(down_cell, BlockState::Electricity(_)) {
                                        new_game_map.cells[y * self.width + x] = BlockState::Copper;
                                        continue
                                    }
                                }
                            }
                            Direction::Down => {
                                if let Some(up_cell) = self.get_cell(x as i32, y as i32 + 1) {
                                    if matches!(up_cell, BlockState::Electricity(Direction::Up))
                                        || !matches!(up_cell, BlockState::Electricity(_)) {
                                        new_game_map.cells[y * self.width + x] = BlockState::Copper;
                                        continue
                                    }
                                }
                            }
                            Direction::Left => {
                                if let Some(right_cell) = self.get_cell(x as i32 + 1, y as i32) {
                                    if matches!(right_cell, BlockState::Electricity(Direction::Right))
                                        || !matches!(right_cell, BlockState::Electricity(_)) {
                                        new_game_map.cells[y * self.width + x] = BlockState::Copper;
                                        continue
                                    }
                                }
                            }
                            Direction::Right => {
                                if let Some(left_cell) = self.get_cell(x as i32 - 1, y as i32) {
                                    if matches!(left_cell, BlockState::Electricity(Direction::Left))
                                        || !matches!(left_cell, BlockState::Electricity(_)) {
                                        new_game_map.cells[y * self.width + x] = BlockState::Copper;
                                        continue
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        
        new_game_map
    }
}

#[derive(Resource)]
struct UpdateTimer(Timer);

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