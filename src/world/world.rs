use crate::plugins::config::Config;
use crate::plugins::control::get_brush_size;
use crate::plugins::window::CustomUiData;
pub(crate) use crate::world::blockstate::BlockState;
use crate::world::drew_game_map::DrewBlocks;
use crate::world::game_map::GameMap;
use bevy::color::palettes::css::WHITE;
use bevy::math::I64Vec2;
use bevy::prelude::MouseButton::Left;
use bevy::prelude::*;
use bevy::ui_widgets::SliderValue;
use bevy_vector_shapes::Shape2dPlugin;
use bevy_vector_shapes::painter::ShapePainter;

pub(crate) struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, draw_preview.after(TransformSystems::Propagate));
        app.add_plugins(Shape2dPlugin::default());
        app.insert_resource(DrewBlocks { block_data: Vec::new()});
        app.insert_resource(CurrentGameState {state: GameState::Drawing});
        app.insert_resource(UpdateTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
        app.add_systems(Update, draw_blocks);
        app.add_systems(Update, update_blocks);
    }
}


fn draw_preview(
    popups: Query<Entity, With<crate::plugins::window::Popup>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut gizmos: Gizmos,
    config: Res<Config>,
    sel_state: Res<SelectedState>,
    game_state: Res<CurrentGameState>,
    sliders: Query<(&SliderValue,&CustomUiData)>
) {
    if popups.iter().count() > 0 {
        return;
    }
    if !config.draw_preview { return; }
    if game_state.state != GameState::Drawing {return;}

    let (camera, camera_transform) = *camera_query;

    if let Some(cursor_position) = window.cursor_position()
        && let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position)

    {
        match sel_state.state {
            Selection::Block(state) => {
                for pos in get_positions_in_range(world_pos.floor().as_i64vec2(), get_brush_size(sliders)-1) {
                    state.draw_preview(pos.as_vec2(), &mut gizmos)
                }
            }
            Selection::Selection(first, maybe_second) => {
                let end = match maybe_second {
                    Some(second) => second,
                    None => world_pos.floor().as_i64vec2(),
                };
                let (center, size) = selection_rect(first, end);
                gizmos.rect_2d(center, size, WHITE);
            }
        }

    }
}
fn selection_rect(a: I64Vec2, b: I64Vec2) -> (Vec2, Vec2) {
    let min = a.min(b).as_vec2();
    let max = (a.max(b) + I64Vec2::ONE).as_vec2();
    ((min + max) * 0.5, max - min)
}
fn draw_blocks(mut painter: ShapePainter,
               blocks: Res<DrewBlocks>,
               mut gizmos: Gizmos,
               config : Res<Config>,
               game_state: Res<CurrentGameState>,
               camera_query: Single<(&Camera, &GlobalTransform,&mut Projection)>,
               window: Single<&Window>) {
    match &game_state.state {
        GameState::Drawing => {
            for block_data in &blocks.block_data {
                let state = block_data.2;
                state.draw(Vec2::new(block_data.0 as f32, block_data.1 as f32), &mut gizmos, &mut painter,config.draw_arrows);
            }
        }

        GameState::Compiling => {}
        GameState::Executing(game_map) => {
            let (camera,gt,mut projection) = camera_query.into_inner();
            let Some((min,max)) = visible_bounds(camera, gt, *window) else {return};
            let Projection::Orthographic(ortho) = &mut *projection else {return};
            game_map.draw(min,max,&mut painter,&mut gizmos, config.draw_arrows && ortho.scale < 0.05);
        }
        GameState::Paused(game_map) => {
            let (camera,gt,mut projection) = camera_query.into_inner();
            let Some((min,max)) = visible_bounds(camera, gt, *window) else {return};
            let Projection::Orthographic(ortho) = &mut *projection else {return};

            game_map.draw(min,max,&mut painter,&mut gizmos, config.draw_arrows && ortho.scale < 0.05);
        }
    }


}

#[derive(Resource)]
pub struct SelectedState{
    pub state: Selection
}

#[derive(Copy, Clone,PartialEq,Eq)]
pub enum Selection{
    Block(BlockState),
    Selection(I64Vec2,Option<I64Vec2>)
}

#[derive(Resource)]
pub struct CurrentGameState {
    pub(crate) state: GameState,
}

#[derive(Clone,PartialEq,Eq)]
pub enum GameState {
    Drawing,
    Compiling,
    Executing(GameMap),
    Paused(GameMap),
}

#[derive(Resource)]
pub(crate) struct UpdateTimer(pub(crate) Timer);

const MAX_STEPS_PER_FRAME: u32 = 512;
fn update_blocks(mut game_state: ResMut<CurrentGameState>,
                 mut timer: ResMut<UpdateTimer>,
                 time: Res<Time>,
                 mouse_button_input: Res<ButtonInput<MouseButton>>,
                 window: Single<&Window>,
                 camera_query: Single<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = camera_query.into_inner();
    let cursor_world = if mouse_button_input.pressed(Left) {
        window
            .cursor_position()
            .and_then(|c| camera.viewport_to_world_2d(camera_transform, c).ok())
    } else {
        None
    };

    {
        let gs = game_state.bypass_change_detection();
        if let GameState::Executing(game) | GameState::Paused(game) = &mut gs.state {
            let cell = cursor_world.and_then(|p| {
                game.to_game_pos(p.x.floor() as i64, p.y.floor() as i64)
            });
            game.set_held(cell);
        }
    }

    let steps = timer.0
        .tick(time.delta())
        .times_finished_this_tick()
        .min(MAX_STEPS_PER_FRAME);

    if steps > 0 {
        if let GameState::Executing(game_map) = &mut game_state.state {
            game_map.step_n(steps);
        }
    }
}

pub fn get_positions_in_range(center: I64Vec2, max_distance: i64) -> Vec<I64Vec2> {
    let mut points = Vec::new();

    for x in -max_distance..=max_distance {
        for y in -max_distance..=max_distance {
            let point = center + I64Vec2::new(x, y);

            if center.distance_squared(point) <= (max_distance*max_distance) {
                points.push(point);
            }
        }
    }
    points
}

pub fn visible_bounds(camera: &Camera, tf: &GlobalTransform, window: &Window) -> Option<(I64Vec2, I64Vec2)> {
    let a = camera.viewport_to_world_2d(tf, Vec2::ZERO).ok()?;
    let b = camera.viewport_to_world_2d(tf, window.size()).ok()?;
    Some((
        a.min(b).floor().as_i64vec2() - I64Vec2::ONE,
        a.max(b).ceil().as_i64vec2() + I64Vec2::ONE,
    ))
}