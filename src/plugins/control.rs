use crate::plugins::config::Config;
use crate::plugins::ui_elements::{selection_popup, text_popup};
use crate::plugins::window::{CustomUiData, EasyClosablePopup};
use crate::world::direction::Direction;
use crate::world::drew_game_map::{DrewBlocks, DrewBlocksFile};
use crate::world::game_map::GameMap;
use crate::world::world;
use crate::world::world::Selection::Block;
use crate::world::world::{
    BlockState, CurrentGameState, GameState, SelectedState, Selection, UpdateTimer,
};
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::math::I64Vec2;
use bevy::math::ops::powf;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, SliderValue, observe};
use bevy::{picking::hover::Hovered, ui::Pressed};
use std::sync::Mutex;
use std::time::Duration;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, start_game);
        app.insert_resource(SelectedState {
            state: Block(BlockState::Copper),
        });
        app.add_systems(
            Update,
            (
                controls,
                slider_system,
                left_mouse_click,
                middle_mouse_click,
                right_mouse_click,
            ),
        );
        app.add_systems(Startup, setup_clipboard);
    }
}

// fn setup(mut commands: Commands) {
//
// }

const MOUSE_SENSITIVITY: f32 = 1f32;
pub const HELP_MESSAGE: &'static str = "\
F1 - Help Popup
F5 - Save
F9 - Load
1-9 - Different blocks
LMB - Place block
MMB - Pick block
RMB - Select region
Ctrl+C - Copy selected region to clipboard
Ctrl+V - Paste region from clipboard to mouse or selection
Shift - Draw Paste preview
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
    mut game_state: ResMut<CurrentGameState>,
    mut selection: ResMut<SelectedState>,
    mut blocks: ResMut<DrewBlocks>,
    mut commands: Commands,
    clipboard: Option<Res<ClipboardRes>>,
    window: Single<&Window>,
    camera_query_2: Single<(&Camera, &GlobalTransform)>,
    mut gizmos: Gizmos
) {
    if popups.iter().count() > 0 {
        return;
    }

    let (mut transform, mut projection) = camera_query.into_inner();

    if let Projection::Orthographic(projection2d) = &mut *projection {
        let ctrl = input.pressed(KeyCode::ControlLeft);

        if accumulated_mouse_scroll.delta != Vec2::ZERO {
            let delta = accumulated_mouse_scroll.delta;
            if ctrl {
                if delta.y < 0.0 {
                    projection2d.scale *=
                        powf(4.0f32, time.delta_secs() * MOUSE_SENSITIVITY * 2f32);
                } else {
                    projection2d.scale *=
                        powf(0.25f32, time.delta_secs() * MOUSE_SENSITIVITY * 2f32);
                }
            } else {
                if let Block(state) = selection.state {
                    if delta.y < 0.0 {
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
                GameState::Drawing => {
                    game_state.state = GameState::Compiling;
                }
                GameState::Compiling => {}
                GameState::Executing(game) => {
                    game_state.state = GameState::Paused(game.clone());
                }
                GameState::Paused(game) => {
                    game_state.state = GameState::Executing(game.clone());
                }
            }
        }
        if input.just_pressed(KeyCode::Escape) {
            match &game_state.state {
                GameState::Drawing => {}
                GameState::Compiling => {
                    game_state.state = GameState::Drawing;
                }
                GameState::Executing(_) => {
                    game_state.state = GameState::Drawing;
                }
                GameState::Paused(_) => {
                    game_state.state = GameState::Drawing;
                }
            }
            if let Selection::Selection(_, _) = &selection.state {
                selection.state = Block(BlockState::Empty);
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
            selection.state = Block(BlockState::Bridge([false; 4]));
        }

        if input.just_pressed(KeyCode::F5) {
            if let Err(e) = blocks.save("save.json") {
                error!("Save error: {e}");
            }
        }
        if input.just_pressed(KeyCode::F9) {
            let loaded = DrewBlocks::load("save.json");
            match loaded {
                Ok(save) => {
                    blocks.block_data = save.block_data;
                }
                Err(e) => {
                    error!("Load error: {e}");
                }
            }
        }
        if input.just_pressed(KeyCode::F1) {
            commands.spawn(text_popup(HELP_MESSAGE));
        }

        if ctrl && input.just_pressed(KeyCode::KeyC) {
            copy_selection_to_clipboard(&Res::from(blocks), &clipboard, &selection);
            if let Selection::Selection(_, Some(_)) = &selection.state {
                selection.state = Block(BlockState::Empty);
            }
            return;
        }

        if ctrl && input.just_pressed(KeyCode::KeyV) {
            paste_from_clipboard(&mut blocks, &clipboard, &selection, window, camera_query_2,&game_state.state);
            if let Selection::Selection(_, _) = &selection.state {
                selection.state = Block(BlockState::Empty);
            }
            return;
        }

        if input.pressed(KeyCode::ShiftLeft) {
            let Some(clipboard) = clipboard else { return };
            if game_state.state != GameState::Drawing {return;}

            match clipboard.0.lock().unwrap().get_text() {
                Ok(text) => {
                    let Ok(file) = serde_json::from_str::<DrewBlocksFile>(&text) else {
                        error!("Cannot paste: invalid JSON");
                        return;
                    };
                    let anchor = match get_paste_anchor(&mut selection, window, camera_query_2) {
                        Some(value) => value,
                        None => return,
                    };

                    for (x, y, state) in DrewBlocks::from(file).block_data {
                        state.draw_preview(Vec2::new((x + anchor.x) as f32, (y + anchor.y) as f32), &mut gizmos)
                    }
                }
                Err(e) => {
                    error!("Cannot read clipboard: {}", e);
                    return;
                }
            }
        }
    }
}

fn get_paste_anchor(selection: &SelectedState, window: Single<&Window>, camera_query_2: Single<(&Camera, &GlobalTransform)>) -> Option<I64Vec2> {
    let anchor: I64Vec2 = match selection.state {
        Selection::Selection(first, Some(second)) => {
            I64Vec2::new(first.x.min(second.x), first.y.max(second.y))
        }
        _ => {
            let (camera, camera_transform) = camera_query_2.into_inner();
            let Some(cursor_position) = window.cursor_position() else {
                return None;
            };
            let Ok(world_pos) =
                camera.viewport_to_world_2d(camera_transform, cursor_position)
            else {
                return None;
            };
            world_pos.floor().as_i64vec2()
        }
    };
    Some(anchor)
}

fn left_mouse_click(
    popups: Query<Entity, With<crate::plugins::window::Popup>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    sel_state: Res<SelectedState>,
    pressed_query: Query<(&Hovered, Has<Pressed>)>,
    game_state: Res<CurrentGameState>,
    window: Single<&Window>,
    mut blocks: ResMut<DrewBlocks>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    sliders: Query<(&SliderValue, &CustomUiData)>,
) {
    if popups.iter().count() > 0 {
        return;
    }

    if !mouse_button_input.pressed(MouseButton::Left) {
        return;
    }

    let (camera, camera_transform) = camera_query.into_inner();

    let is_interacting_with_ui = pressed_query.iter().any(|press| press.1 || press.0.get());
    if is_interacting_with_ui {
        return;
    }

    if game_state.state != GameState::Drawing {
        return;
    }

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    if let Block(state) = sel_state.state {
        for pos in world::get_positions_in_range(
            world_pos.floor().as_i64vec2(),
            get_brush_size(sliders) - 1,
        ) {
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
    game_state: Res<CurrentGameState>,
    blocks: Res<DrewBlocks>,
    mut sel_state: ResMut<SelectedState>,
) {
    if popups.iter().count() > 0 {
        return;
    }

    if !mouse_button_input.just_pressed(MouseButton::Middle) {
        return;
    }
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = camera_query.into_inner();

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let x = world_pos.floor().as_i64vec2().x;
    let y = world_pos.floor().as_i64vec2().y;
    match &game_state.state {
        GameState::Drawing => {
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
        GameState::Compiling => {}
        GameState::Executing(game) => {
            if let Some(block) = game.get(x, y) {
                sel_state.state = Block(*block);
            } else {
                sel_state.state = Block(BlockState::Empty);
            }
        }
        GameState::Paused(game) => {
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
    game_state: Res<CurrentGameState>,
    mut sel_state: ResMut<SelectedState>,
    mut commands: Commands,
) {
    if popups.iter().count() > 0 {
        return;
    }

    if mouse_button_input.just_pressed(MouseButton::Right) {
        let GameState::Drawing = game_state.state else {
            return;
        };

        let Some(cursor_position) = window.cursor_position() else {
            return;
        };

        let (camera, camera_transform) = camera_query.into_inner();

        let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
            return;
        };
        sel_state.state = Selection::Selection(world_pos.floor().as_i64vec2(), None);
    } else if mouse_button_input.just_released(MouseButton::Right) {
        if let Selection::Selection(pos, _) = &sel_state.state {
            let GameState::Drawing = game_state.state else {
                return;
            };

            let Some(cursor_position) = window.cursor_position() else {
                return;
            };

            let (camera, camera_transform) = camera_query.into_inner();

            let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position)
            else {
                return;
            };
            sel_state.state = Selection::Selection(*pos, Some(world_pos.floor().as_i64vec2()));
            commands.spawn(selection_popup(cursor_position));
        }
    }
}

pub fn select_block_state(block_state: BlockState) -> impl Bundle {
    observe(
        move |_: On<Activate>, mut selection: ResMut<SelectedState>| {
            selection.state = Block(block_state);
        },
    )
}
pub fn toggle_config(f: fn(&mut Config) -> &mut bool) -> impl Bundle {
    observe(move |_: On<Activate>, mut config: ResMut<Config>| {
        let flag = f(&mut config);
        *flag = !*flag;
    })
}

pub fn spawn<B: Bundle>(make: impl Fn() -> B + Send + Sync + 'static) -> impl Bundle {
    observe(move |_: On<Activate>, mut commands: Commands| {
        commands.spawn(make());
    })
}

pub fn set_game_state<B: Into<GameState>>(
    game_state: impl Fn(Option<&GameMap>) -> B + Send + Sync + 'static,
) -> impl Bundle {
    observe(
        move |_: On<Activate>, mut selection: ResMut<CurrentGameState>| {
            if let GameState::Executing(map) | GameState::Paused(map) = &selection.state {
                selection.state = game_state(Some(map)).into();
            } else {
                selection.state = game_state(None).into();
            }
        },
    )
}

pub fn copy_selection() -> impl Bundle {
    observe(
        move |_: On<Activate>,
              clipboard: Option<Res<ClipboardRes>>,
              blocks: Res<DrewBlocks>,
              mut selection: ResMut<SelectedState>,
              popups: Query<Entity, With<EasyClosablePopup>>,
              mut commands: Commands| {
            copy_selection_to_clipboard(&blocks, &clipboard, &selection);
            selection.state = Block(BlockState::Empty);
            for e in &popups {
                commands.entity(e).despawn();
            }
        },
    )
}
pub fn paste_selection_or_mouse() -> impl Bundle {
    observe(
        move |_: On<Activate>,
              clipboard: Option<Res<ClipboardRes>>,
              mut blocks: ResMut<DrewBlocks>,
              selection: Res<SelectedState>,
              window: Single<&Window>,
              camera_query: Single<(&Camera, &GlobalTransform)>,
              popups: Query<Entity, With<EasyClosablePopup>>,
              mut commands: Commands,
              game_mode: Res<CurrentGameState>| {
            paste_from_clipboard(&mut blocks, &clipboard, &selection, window, camera_query, &game_mode.state);
            for e in &popups {
                commands.entity(e).despawn();
            }
        },
    )
}

pub fn save_game() -> impl Bundle {
    observe(move |_: On<Activate>, blocks: Res<DrewBlocks>| {
        if let Err(e) = blocks.save("save.json") {
            error!("Save error: {e}");
        }
    })
}
pub fn load_game() -> impl Bundle {
    observe(move |_: On<Activate>, mut blocks: ResMut<DrewBlocks>| {
        let loaded = DrewBlocks::load("save.json");
        match loaded {
            Ok(save) => {
                blocks.block_data = save.block_data;
            }
            Err(e) => {
                error!("Load error: {e}");
            }
        }
    })
}

pub fn get_brush_size(sliders: Query<(&SliderValue, &CustomUiData)>) -> i64 {
    for slider in sliders {
        if slider.1.name == "Size" {
            return slider.0.0.round() as i64;
        }
    }
    1
}

fn slider_system(
    popups: Query<Entity, With<crate::plugins::window::Popup>>,
    input: Res<ButtonInput<KeyCode>>,
    mut update_timer: ResMut<UpdateTimer>,
    changed_sliders: Query<(&SliderValue, &CustomUiData), Changed<SliderValue>>,
) {
    if popups.iter().count() > 0 && !input.pressed(KeyCode::Escape) {
        return;
    }
    for slider_value in &changed_sliders {
        match slider_value.1.name.as_str() {
            "Speed" => {
                update_timer.0.set_duration(Duration::from_secs_f32(
                    1.0 / (slider_value.0.0 + 1.0).powi(10),
                ));
            }
            _ => {}
        }
    }
}

fn start_game(mut game_state: ResMut<CurrentGameState>, blocks: Res<DrewBlocks>) {
    if game_state.state != GameState::Compiling {
        return;
    }

    if blocks.block_data.is_empty() {
        game_state.state = GameState::Drawing;
        return;
    }

    let min_x = blocks
        .block_data
        .iter()
        .filter(|(_, _, state)| *state != BlockState::Empty)
        .map(|(x, _, _)| x)
        .min()
        .unwrap();
    let min_y = blocks
        .block_data
        .iter()
        .filter(|(_, _, state)| *state != BlockState::Empty)
        .map(|(_, y, _)| y)
        .min()
        .unwrap();

    let max_x = blocks
        .block_data
        .iter()
        .filter(|(_, _, state)| *state != BlockState::Empty)
        .map(|(x, _, _)| x)
        .max()
        .unwrap();
    let max_y = blocks
        .block_data
        .iter()
        .filter(|(_, _, state)| *state != BlockState::Empty)
        .map(|(_, y, _)| y)
        .max()
        .unwrap();

    let width = (max_x - min_x + 1) as u64;
    let height = (max_y - min_y + 1) as u64;

    let mut game_map = GameMap::new(width, height, *min_x, *min_y);

    for (x, y, state) in &blocks.block_data {
        if *state == BlockState::Empty {
            continue;
        }
        game_map.set(*x, *y, state.clone());
    }

    game_state.state = GameState::Executing(game_map);
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
    let Selection::Selection(first, Some(second)) = selection.state else {
        return;
    };

    let min = first.min(second);
    let max = first.max(second);

    let file = DrewBlocksFile::blocks_to_state_group(
        &blocks
            .block_data
            .iter()
            .filter(|(x, y, state)| {
                *state != BlockState::Empty
                    && (min.x..=max.x).contains(x)
                    && (min.y..=max.y).contains(y)
            })
            .map(|(x, y, state)| (*x - min.x, *y - max.y, *state)) // относительно левого верхнего угла
            .collect::<Vec<_>>(),
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
    game_mode: &GameState
) {
    let Some(clipboard) = clipboard else { return };
    if *game_mode != GameState::Drawing {return;}
    match clipboard.0.lock().unwrap().get_text() {
        Ok(text) => {
            let Ok(file) = serde_json::from_str::<DrewBlocksFile>(&text) else {
                error!("Cannot paste: invalid JSON");
                return;
            };
            let anchor: I64Vec2 = match get_paste_anchor(selection, window, camera_query) {
                Some(value) => value,
                None => return,
            };

            for (x, y, state) in DrewBlocks::from(file).block_data {
                blocks.add_or_replace_block(state, x + anchor.x, y + anchor.y);
            }
        }
        Err(e) => {
            error!("Cannot read clipboard: {}", e);
            return;
        }
    }
}
