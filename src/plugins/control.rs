use crate::plugins::config::Config;
use crate::plugins::window::{CustomUiData, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::world::direction::Direction;
use crate::world::world::{BlockState, Blocks, GameState, GameStates, SelectedState, UpdateTimer};
use crate::world::{game_map, world};
use bevy::color::palettes::basic::RED;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::math::ops::powf;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::SliderValue;
use std::time::Duration;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, start_game);
        app.insert_resource(SelectedState{ state: BlockState::Copper , size: 1});
        app.add_systems(Update, (controls, button_system, slider_system, left_mouse_click,middle_mouse_click));
    }
}

// fn setup(mut commands: Commands) {
//
// }



const MOUSE_SENSITIVITY: f32 = 1f32;

fn controls(
    camera_query: Single<(&mut Transform, &mut Projection)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    accumulated_mouse_scroll: Res<AccumulatedMouseScroll>,
    mut game_state: ResMut<GameState>,
    mut sel_state: ResMut<SelectedState>,
) {

    let (mut transform, mut projection)
        = camera_query.into_inner();

    if let Projection::Orthographic(projection2d) = &mut *projection {
        if accumulated_mouse_scroll.delta != Vec2::ZERO {
            let delta = accumulated_mouse_scroll.delta;
            if input.pressed(KeyCode::ControlLeft) {
                if delta.y < 0.0 {
                    projection2d.scale *= powf(4.0f32, time.delta_secs() * MOUSE_SENSITIVITY * 2f32);
                }else {
                    projection2d.scale *= powf(0.25f32, time.delta_secs() * MOUSE_SENSITIVITY * 2f32);
                }
            } else {
                if delta.y < 0.0  {
                    sel_state.state = sel_state.state.rotate(true);
                } else {
                    sel_state.state = sel_state.state.rotate(false);
                }
            }
        }

        let speed = 600.0 * time.delta_secs() * projection2d.scale;
        // Camera movement controls
        if input.pressed(KeyCode::KeyW) {
            transform.translation.y += speed;
        }
        if input.pressed(KeyCode::KeyS) {
            transform.translation.y -= speed;
        }
        if input.pressed(KeyCode::KeyA) {
            transform.translation.x -= speed;
        }
        if input.pressed(KeyCode::KeyD) {
            transform.translation.x += speed;
        }
        if input.just_pressed(KeyCode::Space) {
            match &game_state.state {
                GameStates::Drawing => {game_state.state = GameStates::Compiling; }
                GameStates::Compiling => {}
                GameStates::Executing(game) => {game_state.state = GameStates::Paused(game.clone());}
                GameStates::Paused(game) => {game_state.state = GameStates::Executing(game.clone());}
            }
        }
        if input.just_pressed(KeyCode::Escape) {
            match &game_state.state {
                GameStates::Drawing => {}
                GameStates::Compiling => {game_state.state = GameStates::Drawing;}
                GameStates::Executing(_) => {game_state.state = GameStates::Drawing;}
                GameStates::Paused(_) => {game_state.state = GameStates::Drawing;}
            }
        }
        if input.just_pressed(KeyCode::Digit1) {
            sel_state.state = BlockState::Empty;
        }
        if input.just_pressed(KeyCode::Digit2) {
            sel_state.state = BlockState::Copper;
        }
        if input.just_pressed(KeyCode::Digit3) {
            sel_state.state = BlockState::Electricity([false,false,true,false]);
        }
        if input.just_pressed(KeyCode::Digit4) {
            sel_state.state = BlockState::Diode(Direction::Right,false);
        }
    }
}

fn left_mouse_click(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    sel_state: Res<SelectedState>,
    interaction_query: Query<
        (
            &Interaction,
        )
    >,
    hover_query: Query<
        (
            &Hovered,
        )
    >,
    game_state: Res<GameState>,
    window: Single<&Window>,
    mut blocks: ResMut<Blocks>,
    camera_query: Single<(&Camera, &GlobalTransform)>,

) {
    if !mouse_button_input.pressed(MouseButton::Left) {
        return;
    }

    let (camera,camera_transform)
        = camera_query.into_inner();

    let is_interacting_with_ui = interaction_query
        .iter()
        .any(|(interaction,)| *interaction != Interaction::None)
        || hover_query.iter().any(|(hovered,)| hovered.0);
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

    for pos in world::get_positions_in_range(world_pos.floor().as_ivec2(), sel_state.size-1) {
        let x = pos.x;
        let y = pos.y;

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
}

fn middle_mouse_click(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    game_state: Res<GameState>,
    blocks: Res<Blocks>,
    mut sel_state: ResMut<SelectedState>,

){
    if !mouse_button_input.just_pressed(MouseButton::Middle) {
        return;
    }
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let (camera,camera_transform)
        = camera_query.into_inner();

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };
    let x = world_pos.floor().as_ivec2().x;
    let y = world_pos.floor().as_ivec2().y;
    match &game_state.state {
        GameStates::Drawing => {
            if let Some(existing_block) = blocks
                .block_data
                .iter()
                .find(|(bx, by, _)| *bx == x && *by == y)
            {
                sel_state.state = existing_block.2;
            } else {
                sel_state.state = BlockState::Empty;
            }
        }
        GameStates::Compiling => {}
        GameStates::Executing(game) => {
            if let Some(block) = game.get(x, y) {
                sel_state.state = *block;
            } else {
                sel_state.state = BlockState::Empty;
            }
        }
        GameStates::Paused(game) => {
            if let Some(block) = game.get(x, y) {
                sel_state.state = *block;
            } else {
                sel_state.state = BlockState::Empty;
            }
        }
    }

}


fn button_system(
    mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
    (Entity, &Interaction, &mut BackgroundColor, &mut BorderColor, &mut Button, &CustomUiData),
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
                    "Toggle Preview" => config.draw_grid = !config.draw_grid,
                    "Toggle Arrows" => config.draw_arrows = !config.draw_arrows,
                    "Rubber" => selection.state = BlockState::Empty,
                    "Copper" => selection.state = BlockState::Copper,
                    "Electricity" => selection.state = BlockState::Electricity([false,false,true,false]),
                    "Diode" => selection.state = BlockState::Diode(Direction::Right,false),
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
                    "Button" => selection.state = BlockState::Button(false),
                    other => {panic!("Unknown button: {}", other)}
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

fn slider_system(
    mut update_timer: ResMut<UpdateTimer>,
    mut selected_state:  ResMut<SelectedState>,
    changed_sliders: Query<(&SliderValue,&CustomUiData), Changed<SliderValue>>,
) {
    for slider_value in &changed_sliders {
        match slider_value.1.name.as_str() {
            "Speed" => {update_timer.0.set_duration(Duration::from_secs_f32(
                1.0 / (slider_value.0.0 + 1.0).powi(10)
            ));
            }
            "Size" => {
                selected_state.size = slider_value.0.0.round() as i32;
            }
            _ => {}
        }

    }
}

fn start_game(mut game_state: ResMut<GameState>,
        blocks: Res<Blocks>) {
    if game_state.state != GameStates::Compiling { return; }

    if blocks.block_data.is_empty() {
        game_state.state = GameStates::Drawing;
        return;
    }

    let min_x = blocks.block_data.iter()
        .filter(|(_,_,state)| *state != BlockState::Empty).map(|(x,_,_)| x).min().unwrap();
    let min_y = blocks.block_data.iter()
        .filter(|(_,_,state)| *state != BlockState::Empty).map(|(_,y,_)| y).min().unwrap();

    let max_x = blocks.block_data.iter()
        .filter(|(_,_,state)| *state != BlockState::Empty).map(|(x,_,_)| x).max().unwrap();
    let max_y = blocks.block_data.iter()
        .filter(|(_,_,state)| *state != BlockState::Empty).map(|(_,y,_)| y).max().unwrap();


    let width = (max_x - min_x + 1) as usize;
    let height = (max_y - min_y + 1) as usize;

    let mut game_map = game_map::GameMap::new(width, height, *min_x, *min_y);

    for (x, y, state) in &blocks.block_data {
        if *state == BlockState::Empty { continue; }
        game_map.set(*x, *y, state.clone());
    }

    game_state.state = GameStates::Executing(game_map);
}

