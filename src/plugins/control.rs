use crate::plugins::config::Config;
use crate::plugins::window::{ButtonData, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::plugins::world::{BlockState, Blocks, Direction, GameState, GameStates, SelectedState};
use bevy::color::palettes::basic::RED;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::math::ops::powf;
use bevy::prelude::*;
use crate::plugins::world;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, (controls,start_game));
        app.insert_resource(SelectedState{ state: BlockState::Copper });
        app.add_systems(Update, (update_button_visibility, button_system));
    }
}

// fn setup(mut commands: Commands) {
//
// }



const MOUSE_SENSITIVITY: f32 = 1f32;

fn controls(
    camera_query: Single<(&Camera, &mut Transform, &mut Projection,&GlobalTransform)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Fixed>>,
    accumulated_mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    mut blocks: ResMut<Blocks>,
    sel_state: Res<SelectedState>,
    interaction_query: Query<
        (
            &Interaction,
        )
    >,
    game_state: Res<GameState>

) {

    let (camera, mut transform, mut projection,camera_transform)
        = camera_query.into_inner();

    on_mouse_click(&mouse_button_input, window, &mut blocks, &interaction_query, camera, camera_transform, sel_state, game_state);

    if mouse_button_input.just_pressed(MouseButton::Left) {

    }

    if mouse_button_input.just_released(MouseButton::Left) {

    }
    





    // Camera zoom controls
    if let Projection::Orthographic(projection2d) = &mut *projection {
        if accumulated_mouse_scroll.delta != Vec2::ZERO {
            let delta = accumulated_mouse_scroll.delta;
            if delta.y < 0f32 {
                projection2d.scale *= powf(4.0f32, time.delta_secs() * MOUSE_SENSITIVITY * 2f32);
            } else {
                projection2d.scale *= powf(0.25f32, time.delta_secs() * MOUSE_SENSITIVITY * 2f32);
            }
        }

        let fspeed = 600.0 * time.delta_secs() * projection2d.scale;
        // Camera movement controls
        if input.pressed(KeyCode::KeyW) {
            transform.translation.y += fspeed;
        }
        if input.pressed(KeyCode::KeyS) {
            transform.translation.y -= fspeed;
        }
        if input.pressed(KeyCode::KeyA) {
            transform.translation.x -= fspeed;
        }
        if input.pressed(KeyCode::KeyD) {
            transform.translation.x += fspeed;
        }
    }
}

fn on_mouse_click(
    mouse_button_input: &Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    blocks: &mut ResMut<Blocks>,
    interaction_query: &Query<(&Interaction,)>,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    sel_state: Res<SelectedState>,
    game_state: Res<GameState>,
) {
    if !mouse_button_input.pressed(MouseButton::Left) {
        return;
    }

    let is_interacting_with_ui = interaction_query
        .iter()
        .any(|(interaction,)| *interaction != Interaction::None);
    if is_interacting_with_ui {
        return;
    }

    if game_state.state != GameStates::Drawing {
        return;
    }

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let x = world_pos.x.floor() as i32;
    let y = world_pos.y.floor() as i32;

    if let Some(existing_block) = blocks
        .block_data
        .iter_mut()
        .find(|(bx, by, _)| *bx == x && *by == y)
    {
        existing_block.2 = sel_state.state;
    } else {
        blocks.block_data.push((x, y, sel_state.state));
    }
}

fn update_button_visibility(
    game_state: Res<GameState>,
    mut buttons: Query<(&ButtonData, &mut Visibility, &mut Node)>,
) {
    if !game_state.is_changed() {
        return;
    }

    for (button_data, mut visibility, mut node) in &mut buttons {
        let should_show = match (&game_state.state, button_data.name.as_str()) {
            (GameStates::Drawing, "Compile") => true,
            (GameStates::Executing(_), "Terminate" | "Pause") => true,
            (GameStates::Paused(_), "Terminate" | "Resume") => true,
            (_,"Compile" |"Terminate" | "Pause" | "Resume") => false,
            _ => true,
        };

        *visibility = if should_show { Visibility::Visible } else { Visibility::Hidden };
        node.display = if should_show { Display::Flex } else { Display::None };
    }
}

fn button_system(
    mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
    (Entity, &Interaction, &mut BackgroundColor, &mut BorderColor, &mut Button, &ButtonData),
    Changed<Interaction>,
    >,
    mut config: ResMut<Config>,
    mut selection: ResMut<SelectedState>,
    mut game_state: ResMut<GameState>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, button_data) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity, FocusCause::Pressed);
                *color = PRESSED_BUTTON.into();
                *border_color = BorderColor::all(RED);

                match button_data.name.as_str() {
                    "Toggle Grid" => config.draw_grid = !config.draw_grid,
                    "Toggle Arrows" => config.draw_arrows = !config.draw_arrows,
                    "Rubber" => selection.state = BlockState::Empty,
                    "Copper" => selection.state = BlockState::Copper,
                    "Electricity Right" => selection.state = BlockState::Electricity(Direction::Right),
                    "Electricity Left" => selection.state = BlockState::Electricity(Direction::Left),
                    "Electricity Up" => selection.state = BlockState::Electricity(Direction::Up),
                    "Electricity Down" => selection.state = BlockState::Electricity(Direction::Down),
                    "Compile" => game_state.state = GameStates::Compiling,
                    "Terminate" => game_state.state = GameStates::Drawing,
                    "Pause" => {
                        if let GameStates::Executing(game_map) = &game_state.state {
                            game_state.state = GameStates::Paused(game_map.clone());
                        }
                    }
                    "Resume" => {
                        if let GameStates::Paused(game_map) = &game_state.state {
                            game_state.state = GameStates::Executing(game_map.clone());
                        }
                    }
                    _ => {}
                }
                button.set_changed();
            }
            Interaction::Hovered => {
                input_focus.set(entity, FocusCause::Pressed);
                *color = HOVERED_BUTTON.into();
                *border_color = BorderColor::all(Color::WHITE);
                button.set_changed();
            }
            Interaction::None => {
                input_focus.clear();
                *color = NORMAL_BUTTON.into();
                *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}

fn start_game(mut game_state: ResMut<GameState>,
        blocks: Res<Blocks>) {
    if game_state.state != GameStates::Compiling { return; }

    let min_x = blocks.block_data.iter().map(|(x,_,_)| x).min().unwrap();
    let min_y = blocks.block_data.iter().map(|(_,y,_)| y).min().unwrap();

    let max_x = blocks.block_data.iter().map(|(x,_,_)| x).max().unwrap();
    let max_y = blocks.block_data.iter().map(|(_,y,_)| y).max().unwrap();


    let width = (max_x - min_x + 1) as usize;
    let height = (max_y - min_y + 1) as usize;

    let mut game_map = world::GameMap::new(width, height, *min_x, *min_y);

    for (x, y, state) in &blocks.block_data {
        game_map.set(*x, *y, state.clone());
    }

    game_state.state = GameStates::Executing(game_map);
}

