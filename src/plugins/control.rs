use std::sync::Mutex;
use crate::plugins::config::Config;
use crate::plugins::window::{help_popup, CustomUiData, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON, selection_popup};
use crate::world::direction::Direction;
use crate::world::world::{BlockState, GameState, GameStates, SelectedState, Selection, UpdateTimer};
use crate::world::{game_map, world};
use bevy::color::palettes::basic::RED;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::math::ops::powf;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::SliderValue;
use std::time::Duration;
use crate::world::drew_game_map::{DrewBlocks, DrewBlocksFile};
use crate::world::world::Selection::{Block};

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, start_game);
        app.insert_resource(SelectedState{ state: Block(BlockState::Copper)});
        app.add_systems(
            Update,
            (controls, button_system, slider_system, left_mouse_click,middle_mouse_click,right_mouse_click)
        );
        app.add_systems(Startup, setup_clipboard);
    }
}

// fn setup(mut commands: Commands) {
//
// }



const MOUSE_SENSITIVITY: f32 = 1f32;
const HELP_MESSAGE: &'static str = "\
F1 - Help Popup
F5 - Save
F9 - Load
1-9 - Different blocks
LMB - Place block
MMB - Pick block
RMB - Select region
Ctrl+C - Copy selected region to clipboard
Ctrl+V - Paste region from clipboard to mouse or selection
Scroll - Rotate block
Ctrl+Scroll - Zoom
WASD - Move camera
SPACE - Compile/Pause/Resume
ESC - Terminate(back to drawing)
";

fn controls(
    popups: Query<Entity, With<crate::plugins::window::Popup>>,
    camera_query: Single<(&mut Transform, &mut Projection)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    accumulated_mouse_scroll: Res<AccumulatedMouseScroll>,
    mut game_state: ResMut<GameState>,
    mut selection: ResMut<SelectedState>,
    mut blocks: ResMut<DrewBlocks>,
    mut commands: Commands,
    clipboard: Option<Res<ClipboardRes>>,
    window: Single<&Window>,
    camera_query_2: Single<(&Camera, &GlobalTransform)>,
) {
    if popups.iter().count() > 0 {
        return;
    }

    let (mut transform, mut projection)
        = camera_query.into_inner();

    if let Projection::Orthographic(projection2d) = &mut *projection {
        let ctrl = input.pressed(KeyCode::ControlLeft);

        if accumulated_mouse_scroll.delta != Vec2::ZERO {
            let delta = accumulated_mouse_scroll.delta;
            if ctrl {
                if delta.y < 0.0 {
                    projection2d.scale *= powf(4.0f32, time.delta_secs() * MOUSE_SENSITIVITY * 2f32);
                }else {
                    projection2d.scale *= powf(0.25f32, time.delta_secs() * MOUSE_SENSITIVITY * 2f32);
                }
            } else {
                if let Block(state) = selection.state {
                    if delta.y < 0.0  {
                        selection.state = Block(state.rotate(true))
                    } else {
                        selection.state = Block(state.rotate(false))
                    }
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
                selection.state = Block(BlockState::Empty);
            }
            if input.just_pressed(KeyCode::Digit2) {
                selection.state = Block(BlockState::Copper);
            }
            if input.just_pressed(KeyCode::Digit3) {
                selection.state = Block(BlockState::Electricity([false, false, true, false]));
            }
            if input.just_pressed(KeyCode::Digit4) {
                selection.state = Block(BlockState::Diode(Direction::Right, false, false));
            }
            if input.just_pressed(KeyCode::Digit5) {
                selection.state = Block(BlockState::Bridge([false;4]));
            }

        if input.just_pressed(KeyCode::F5) {
            if let Err(e) = blocks.save("save.json") {
                error!("Save error: {e}");
            }
        }
        if input.just_pressed(KeyCode::F9) {
            let loaded = DrewBlocks::load("save.json");
            match loaded {
                Ok(save) => {blocks.block_data = save.block_data;}
                Err(e) => {error!("Load error: {e}");}
            }
        }
        if input.just_pressed(KeyCode::F1) {
            commands.spawn(help_popup(HELP_MESSAGE.into()));
        }

        if ctrl && input.just_pressed(KeyCode::KeyC) {
            copy_selection_to_clipboard(&Res::from(blocks), &clipboard, &selection);
            if let Selection::Selection(_,Some(_)) = &selection.state {
                selection.state = Block(BlockState::Empty);
            }
            return;
        }

        if ctrl && input.just_pressed(KeyCode::KeyV) {
            paste_from_clipboard(&mut blocks, &clipboard, &selection, window, camera_query_2);
            if let Selection::Selection(_,_) = &selection.state {
                selection.state = Block(BlockState::Empty);
            }
            return;
        }
    }
}

fn left_mouse_click(
    popups: Query<Entity, With<crate::plugins::window::Popup>>,
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
    mut blocks: ResMut<DrewBlocks>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    sliders: Query<(&SliderValue,&CustomUiData)>
) {
    if popups.iter().count() > 0 {
        return;
    }

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

    if let Block(state) = sel_state.state {
        for pos in world::get_positions_in_range(world_pos.floor().as_ivec2(), get_brush_size(sliders)-1) {
            let x = pos.x;
            let y = pos.y;

            blocks.add_or_replace_block(state, x, y);
        }
    }

}

fn middle_mouse_click(
    popups: Query<Entity, With<crate::plugins::window::Popup>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    game_state: Res<GameState>,
    blocks: Res<DrewBlocks>,
    mut sel_state: ResMut<SelectedState>,

){
    if popups.iter().count() > 0 {
        return;
    }

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
                    sel_state.state = Block(existing_block.2);
                } else {
                    sel_state.state = Block(BlockState::Empty);
                }
            }
            GameStates::Compiling => {}
            GameStates::Executing(game) => {
                if let Some(block) = game.get(x, y) {
                    sel_state.state = Block(*block);
                } else {
                    sel_state.state = Block(BlockState::Empty);
                }
            }
            GameStates::Paused(game) => {
                if let Some(block) = game.get(x, y) {
                    sel_state.state = Block(*block);
                } else {
                    sel_state.state = Block(BlockState::Empty);
                }
            }
        }


}
fn right_mouse_click(
    popups: Query<Entity, With<crate::plugins::window::Popup>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    game_state: Res<GameState>,
    mut sel_state: ResMut<SelectedState>,
    mut commands: Commands,

) {
    if popups.iter().count() > 0 {
        return;
    }

    if mouse_button_input.just_pressed(MouseButton::Right) {
        let GameStates::Drawing = game_state.state else {
            return;
        };

        let Some(cursor_position) = window.cursor_position() else {
            return;
        };

        let (camera,camera_transform)
            = camera_query.into_inner();

        let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
            return;
        };
        sel_state.state = Selection::Selection(world_pos.floor().as_ivec2(),None);
    } else if mouse_button_input.just_released(MouseButton::Right) {
        if let Selection::Selection(pos,_) = &sel_state.state {
            let GameStates::Drawing = game_state.state else {
                return;
            };

            let Some(cursor_position) = window.cursor_position() else {
                return;
            };

            let (camera,camera_transform)
                = camera_query.into_inner();

            let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
                return;
            };
            sel_state.state = Selection::Selection(*pos,Some(world_pos.floor().as_ivec2()));
            commands.spawn(selection_popup(cursor_position));
        }
    }

}


fn button_system(
    popups: Query<Entity, With<crate::plugins::window::Popup>>,
    mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
    (Entity, &Interaction, &mut BackgroundColor, &mut BorderColor, &mut Button, &CustomUiData),
    Changed<Interaction>,
    >,
    mut config: ResMut<Config>,
    mut selection: ResMut<SelectedState>,
    mut game_state: ResMut<GameState>,
    mut blocks: ResMut<DrewBlocks>,
    mut commands: Commands,
    clipboard: Option<Res<ClipboardRes>>,
    window: Single<&Window>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, button_data) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if popups.iter().count() > 0 {
                    return;
                }
                input_focus.set(entity, FocusCause::Pressed);
                *color = PRESSED_BUTTON.into();
                *border_color = BorderColor::all(RED);

                match button_data.name.as_str() {
                    "Toggle Preview" => config.draw_preview = !config.draw_preview,
                    "Toggle Arrows" => config.draw_arrows = !config.draw_arrows,
                    "Rubber" => selection.state = Block(BlockState::Empty),
                    "Copper" => selection.state = Block(BlockState::Copper),
                    "Electricity" => selection.state = Block(BlockState::Electricity([false,false,true,false])),
                    "Diode" => selection.state = Block(BlockState::Diode(Direction::Right,false,false)),
                    "Compile" => game_state.state = GameStates::Compiling,
                    "Terminate" => game_state.state = GameStates::Drawing,
                    "Button" => selection.state = Block(BlockState::Button(false)),
                    "Bridge" => selection.state = Block(BlockState::Bridge([false;4])),
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
                    "Save" => {
                        if let Err(e) = blocks.save("save.json") {
                            error!("Save error: {e}");
                        }
                        return;
                    }
                    "Load" => {
                        let loaded = DrewBlocks::load("save.json");
                        match loaded {
                            Ok(save) => {blocks.block_data = save.block_data;}
                            Err(e) => {error!("Load error: {e}");}
                        }
                        return;
                    },
                    "Help" => { commands.spawn(help_popup(HELP_MESSAGE.into())); }

                    "Copy" => {
                        copy_selection_to_clipboard(&Res::from(blocks), &clipboard, &selection);
                        selection.state = Block(BlockState::Empty);
                        return;
                    }
                    "Paste" => {
                        paste_from_clipboard(&mut blocks, &clipboard, &selection,window,camera_query);
                        selection.state = Block(BlockState::Empty);
                        return;
                    }

                    other => {error!("Unknown button: {}", other)}
                }
                button.set_changed();
            }
            Interaction::Hovered => {
                input_focus.set(entity, FocusCause::Pressed);
                if popups.iter().count() > 0 {
                    return;
                }
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

pub fn get_brush_size(sliders: Query<(&SliderValue, &CustomUiData)>) -> i32 {
    for slider in sliders {
        if slider.1.name == "Size" {
            return slider.0.0.round() as i32;
        }
    }
    1
}

fn slider_system(
    popups: Query<Entity, With<crate::plugins::window::Popup>>,
    input: Res<ButtonInput<KeyCode>>,
    mut update_timer: ResMut<UpdateTimer>,
    changed_sliders: Query<(&SliderValue,&CustomUiData), Changed<SliderValue>>,
) {
    if popups.iter().count() > 0 && !input.pressed(KeyCode::Escape) {
        return;
    }
    for slider_value in &changed_sliders {
        match slider_value.1.name.as_str() {
            "Speed" => {update_timer.0.set_duration(Duration::from_secs_f32(
                1.0 / (slider_value.0.0 + 1.0).powi(10)
            ));
            }
            _ => {}
        }

    }
}

fn start_game(mut game_state: ResMut<GameState>,
        blocks: Res<DrewBlocks>) {
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

#[derive(Resource)]
struct ClipboardRes(Mutex<arboard::Clipboard>);

fn setup_clipboard(mut commands: Commands) {
    match arboard::Clipboard::new() {
        Ok(cb) => commands.insert_resource(ClipboardRes(Mutex::new(cb))),
        Err(e) => error!("буфер обмена недоступен: {e}"),
    }
}

fn copy_selection_to_clipboard(
    blocks: &Res<DrewBlocks>,
    clipboard: &Option<Res<ClipboardRes>>,
    selection: &SelectedState,

) {
    let Some(clipboard) = clipboard else { return };
    let Selection::Selection(first,Some(second)) = selection.state else { return };

    let min = first.min(second);
    let max = first.max(second);

    let file = DrewBlocksFile::blocks_to_state_group(
        &blocks.block_data.iter()
            .filter(|(x, y, state)| {
                *state != BlockState::Empty
                    && (min.x..=max.x).contains(x)
                    && (min.y..=max.y).contains(y)
            })
            .map(|(x, y, state)| (*x - min.x, *y - max.y, *state)) // относительно левого верхнего угла
            .collect::<Vec<_>>()
    );

    match serde_json::to_string(&file) {
        Ok(json) => {
            if let Err(e) = clipboard.0.lock().unwrap().set_text(json) {
                error!("Cannot copy: {e}");
            }
        }
        Err(e) => error!("Cannot copy: {e}"),
    }
}

fn paste_from_clipboard(
    blocks: &mut ResMut<DrewBlocks>,
    clipboard: &Option<Res<ClipboardRes>>,
    selection: &SelectedState,
    window: Single<&Window>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
) {
    let Some(clipboard) = clipboard else { return };

   match clipboard.0.lock().unwrap().get_text() {
       Ok(text) => {
           let Ok(file) = serde_json::from_str::<DrewBlocksFile>(&text) else {
               error!("Cannot paste: invalid JSON");
               return;
           };
           let anchor: IVec2 = match selection.state {
               Selection::Selection(first, Some(second)) => {
                   IVec2::new(first.x.min(second.x), first.y.max(second.y))
               }
               _ => {
                   let (camera, camera_transform) = camera_query.into_inner();
                   let Some(cursor_position) = window.cursor_position() else { return };
                   let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else { return };
                   world_pos.floor().as_ivec2()
               }
           };

           for (x, y, state) in DrewBlocks::from(file).block_data {
               blocks.add_or_replace_block(state, x + anchor.x, y + anchor.y);
           }
       }
       Err(e) => {
           error!("Cannot read clipboard: {}", e);
           return
       }
   }
}

