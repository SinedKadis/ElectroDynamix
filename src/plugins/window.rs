use crate::plugins::control::{HELP_MESSAGE, load_game, save_game, select_block_state, set_game_state, spawn, toggle_config};
use crate::plugins::ui_elements::{button_bundle, slider_bundle, text_popup};
use crate::world::blockstate::BlockState;
use crate::world::direction::Direction;
use crate::world::world::{CurrentGameState, GameState};
use bevy::color::palettes::basic::RED;
use bevy::input_focus::InputFocus;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::{SliderRange, SliderThumb, SliderValue};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFocus>();
        app.add_systems(Startup, setup);
        app.add_systems(PostStartup, setup_camera);
        app.add_systems(
            Update,
            (
                button_system,
                update_ui_elements_visibility,
                update_slider_visuals,
                close_popup,
            ),
        );
    }
}

pub const BACKGROUND_COLOR: Color = Color::linear_rgb(0.01, 0.01, 0.01);
pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(0.0, 0.0, 0.0);

pub const SLIDER_TRACK: Color = Color::srgb(0.15, 0.15, 0.15);
pub const SLIDER_THUMB: Color = Color::srgb(0.25, 0.25, 0.25);

fn setup(mut commands: Commands) {
    // 1. Camera
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(BACKGROUND_COLOR),
            ..default()
        },
    ));

    // 2. Toolbar anchored to the bottom-left in a horizontal row
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(12.0),
            bottom: px(12.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Start,
            column_gap: px(6.0),
            ..default()
        },
        children![
            button_bundle("Rubber", select_block_state(BlockState::Empty)),
            button_bundle("Wall", select_block_state(BlockState::Wall)),
            button_bundle("Copper", select_block_state(BlockState::Copper)),
            button_bundle(
                "Electricity",
                select_block_state(BlockState::Electricity([false, false, true, false]))
            ),
            button_bundle(
                "Diode",
                select_block_state(BlockState::Diode(Direction::Right, false, false))
            ),
            button_bundle("Button",select_block_state(BlockState::Button(false))),
            button_bundle("Bridge",select_block_state(BlockState::Bridge([false;4]))),
        ],
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(12.0),
            top: px(12.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Start,
            row_gap: px(3.0),
            ..default()
        },
        children![
            button_bundle("Help",spawn(|| text_popup(HELP_MESSAGE))),
            button_bundle("Toggle Preview",toggle_config(|c| &mut c.draw_preview)),
            button_bundle("Toggle Arrows",toggle_config(|c| &mut c.draw_arrows))
        ],
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: px(12.0),
            top: px(12.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::End,
            row_gap: px(3.0),
            ..default()
        },
        children![
            button_bundle(String::from("Compile"), set_game_state( |_| GameState::Compiling)),
            button_bundle(String::from("Terminate"), set_game_state( |_| GameState::Drawing)),
            button_bundle(String::from("Pause"),
                set_game_state( |map| GameState::Paused(map.expect("No map found when pausing").clone()))),
            button_bundle(String::from("Resume"),
                set_game_state( |map| GameState::Executing(map.expect("No map found when resuming").clone())))
        ],
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: px(12.0),
            bottom: px(12.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::End,
            row_gap: px(3.0),
            ..default()
        },
        children![
            slider_bundle(String::from("Speed"), 0f32, 1f32, 0f32),
            button_bundle(String::from("Save"),save_game()),
            button_bundle(String::from("Load"),load_game())
        ],
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(12.0),
            bottom: px(60.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(3.0),
            ..default()
        },
        children![slider_bundle(String::from("Size"), 1f32, 5f32, 1f32)],
    ));
}

fn update_ui_elements_visibility(
    game_state: Res<CurrentGameState>,
    mut buttons: Query<(&CustomUiData, &mut Visibility, &mut Node)>,
) {
    if !game_state.is_changed() {
        return;
    }

    for (button_data, mut visibility, mut node) in &mut buttons {
        let should_show = match (&game_state.state, button_data.name.as_str()) {
            (GameState::Drawing, "Compile") => true,
            (GameState::Executing(_), "Terminate" | "Pause" | "Speed") => true,
            (GameState::Paused(_), "Terminate" | "Resume" | "Speed") => true,
            (_, "Compile" | "Terminate" | "Pause" | "Resume" | "Speed") => false,
            _ => true,
        };

        *visibility = if should_show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        node.display = if should_show {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn button_system(
    popups: Query<(), With<Popup>>,
    ez_popups: Query<(), With<EasyClosablePopup>>,
    parents: Query<&ChildOf>,
    mut buttons: Query<
        (Entity, Has<Pressed>, &Hovered, &mut BackgroundColor, &mut BorderColor),
        With<bevy::ui_widgets::Button>,
    >,
) {
    let popup_open = !popups.is_empty();
    let ez_popup_open = !popups.is_empty();

    for (entity, pressed, hovered, mut color, mut border_color) in &mut buttons {
        let blocked =
            (popup_open && !is_inside_popup(entity, &parents, &popups))
                || (ez_popup_open && !is_inside_ez_popup(entity, &parents, &ez_popups));

        let (bg, border): (Color, Color) = if blocked {
            (NORMAL_BUTTON, Color::BLACK)
        } else if pressed {
            (PRESSED_BUTTON, Color::from(RED))
        } else if hovered.get() {
            (HOVERED_BUTTON, Color::WHITE)
        } else {
            (NORMAL_BUTTON, Color::BLACK)
        };

        *color = bg.into();
        *border_color = BorderColor::all(border);
    }
}

fn is_inside_popup(
    mut e: Entity,
    parents: &Query<&ChildOf>,
    popups: &Query<(), With<Popup>>,
) -> bool {
    loop {
        if popups.contains(e) {
            return true;
        }
        match parents.get(e) {
            Ok(parent) => e = parent.parent(),
            Err(_) => return false,
        }
    }
}
fn is_inside_ez_popup(
    mut e: Entity,
    parents: &Query<&ChildOf>,
    popups: &Query<(), With<EasyClosablePopup>>,
) -> bool {
    loop {
        if popups.contains(e) {
            return true;
        }
        match parents.get(e) {
            Ok(parent) => e = parent.parent(),
            Err(_) => return false,
        }
    }
}
fn update_slider_visuals(
    sliders: Query<(&SliderValue, &SliderRange, &Children, &ComputedNode), Changed<SliderValue>>,
    mut thumbs: Query<(&mut Node, &ComputedNode), With<SliderThumb>>,
) {
    for (value, range, children, track_computed) in &sliders {
        let fraction = ((value.0 - range.start()) / (range.end() - range.start())).clamp(0.0, 1.0);
        let track_width = track_computed.size().x - 5.0;

        for &child in children {
            if let Ok((mut node, thumb_computed)) = thumbs.get_mut(child) {
                let thumb_width = thumb_computed.size().x;
                let available = (track_width - thumb_width).max(0.0);
                node.left = Val::Px(fraction * available);
            }
        }
    }
}

fn setup_camera(camera_query: Single<(&mut Camera, &mut Transform, &mut Projection)>) {
    let (_camera, _transform, mut projection) = camera_query.into_inner();

    if let Projection::Orthographic(projection2d) = &mut *projection {
        projection2d.scale = 0.01;
    }
}

#[derive(Component)]
pub struct CustomUiData {
    pub(crate) name: String,
}

#[derive(Component)]
pub struct PopupCloseButton;

#[derive(Component)]
pub struct Popup;
#[derive(Component)]
pub struct EasyClosablePopup;

fn close_popup(
    mut mouse_button_input: ResMut<ButtonInput<MouseButton>>,
    buttons: Query<Has<Pressed>, With<PopupCloseButton>>,
    popups: Query<Entity, With<Popup>>,
    easy_closable_popups: Query<(Entity, &Hovered), With<EasyClosablePopup>>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    let button = buttons.iter().any(|i| i);
    let mouse = mouse_button_input.get_just_pressed().count() > 0;
    let keyboard = input.get_just_pressed().count() > 0;

    for (e, hovered) in &easy_closable_popups {
        if keyboard || (mouse && !hovered.get()) {
            commands.entity(e).despawn();
        }
    }

    if button || input.just_pressed(KeyCode::Escape) {
        for e in &popups {
            commands.entity(e).despawn();
        }
        mouse_button_input.reset_all();
    }
}
