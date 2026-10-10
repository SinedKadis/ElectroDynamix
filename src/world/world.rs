use crate::plugins::config::Config;
pub(crate) use crate::world::blockstate::BlockState;
use crate::world::game_map::GameMap;
use bevy::prelude::*;
use bevy_vector_shapes::Shape2dPlugin;
use bevy_vector_shapes::painter::ShapePainter;

pub(crate) struct WorldPlugin;

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

#[derive(Resource)]
pub(crate) struct UpdateTimer(pub(crate) Timer);

fn update_blocks(mut game_state: ResMut<GameState>,
    mut timer: ResMut<UpdateTimer>,
    time: Res<Time>, 
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let new_game_map : GameMap = match &game_state.state {
            GameStates::Drawing => {return;}
            GameStates::Compiling => {return;}
            GameStates::Executing(game_map) => {game_map.update(mouse_button_input, window, camera_query)}
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