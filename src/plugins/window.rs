use crate::world::world::{GameState, GameStates};
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::ui_widgets::{
    SliderRange, SliderThumb, SliderValue,
};
use crate::plugins::ui_elements::{button_bundle, slider_bundle};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFocus>();
        app.add_systems(Startup, setup);
        app.add_systems(PostStartup, setup_camera);
        app.add_systems(
            Update,
            (
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
            button_bundle(String::from("Rubber")),
            button_bundle(String::from("Copper")),
            button_bundle(String::from("Electricity")),
            button_bundle(String::from("Diode")),
            button_bundle(String::from("Button")),
            button_bundle(String::from("Bridge")),
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
            button_bundle(String::from("Help")),
            button_bundle(String::from("Toggle Preview")),
            button_bundle(String::from("Toggle Arrows"))
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
            button_bundle(String::from("Compile")),
            button_bundle(String::from("Terminate")),
            button_bundle(String::from("Pause")),
            button_bundle(String::from("Resume"))
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
            button_bundle(String::from("Save")),
            button_bundle(String::from("Load"))
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
    game_state: Res<GameState>,
    mut buttons: Query<(&CustomUiData, &mut Visibility, &mut Node)>,
) {
    if !game_state.is_changed() {
        return;
    }

    for (button_data, mut visibility, mut node) in &mut buttons {
        let should_show = match (&game_state.state, button_data.name.as_str()) {
            (GameStates::Drawing, "Compile") => true,
            (GameStates::Executing(_), "Terminate" | "Pause" | "Speed") => true,
            (GameStates::Paused(_), "Terminate" | "Resume" | "Speed") => true,
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
    buttons: Query<&Interaction, (Changed<Interaction>, With<PopupCloseButton>)>,
    popups: Query<Entity, With<Popup>>,
    easy_closable_popups: Query<Entity, With<EasyClosablePopup>>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    let button = buttons.iter().any(|i| *i == Interaction::Pressed);
    let mouse = mouse_button_input.get_just_pressed().count() > 0;
    let keyboard = input.get_just_pressed().count() > 0;

    if keyboard || mouse {
        for e in &easy_closable_popups {
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
