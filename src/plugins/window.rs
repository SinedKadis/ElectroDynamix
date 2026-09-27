use crate::plugins::world::{GameState, GameStates};
use bevy::input_focus::InputFocus;
use bevy::
prelude::*;
use bevy::picking::hover::Hovered;
use bevy::ui_widgets::{observe, slider_self_update, Slider, SliderRange, SliderThumb, SliderValue, TrackClick};

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFocus>();
        app.add_systems(Startup, setup);
        app.add_systems(PostStartup, setup_camera);
        app.add_systems(Update, (update_ui_elements_visibility,
                                 update_slider_visuals));
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
            align_items: AlignItems::Center,
            column_gap: px(6.0),
            ..default()
        },
        children![
            button_bundle(String::from("Rubber")),
            button_bundle(String::from("Copper")),
            button_bundle(String::from("Electricity Right")),
            button_bundle(String::from("Electricity Left")),
            button_bundle(String::from("Electricity Up")),
            button_bundle(String::from("Electricity Down")),
        ],
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(12.0),
            top: px(12.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(3.0),
            ..default()
        },
        children![
            button_bundle(String::from("Toggle Grid")),
            button_bundle(String::from("Toggle Arrows"))
        ],
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: px(12.0),
            top: px(12.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
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
            align_items: AlignItems::Center,
            row_gap: px(3.0),
            ..default()
        },
        children![
            slider_bundle(String::from("Speed"),0f32,1f32,0f32)
        ],
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
            (_,"Compile" |"Terminate" | "Pause" | "Resume" | "Speed") => false,
            _ => true,
        };

        *visibility = if should_show { Visibility::Visible } else { Visibility::Hidden };
        node.display = if should_show { Display::Flex } else { Display::None };
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
        projection2d.scale = 0.1;
    }
}



fn button_bundle(text: String) -> impl Bundle {
    (
        Button,
        Node {
            padding: UiRect::axes(px(10.0), px(6.0)),
            border: UiRect::all(px(2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(6.0)),
            ..default()
        },
        BorderColor::all(Color::WHITE),
        BackgroundColor(NORMAL_BUTTON),
        children![(
            Text::new(&text),
            TextFont {
                font_size: FontSize::Px(13.0),
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextShadow::default(),
        )],
        CustomUiData { name: text},
        Visibility::Visible
    )
}

fn slider_bundle(text: String, min: f32, max: f32, initial: f32) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(4.0),
            ..default()
        },
        children![
            (
                Text::new(text.clone()),
                TextFont { font_size: FontSize::Px(13.0), ..default() },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextShadow::default(),
                CustomUiData { name: text.clone() },
            ),
            (
                Slider {
                    track_click: TrackClick::Snap,
                    ..default()
                },
                SliderValue(initial),
                SliderRange::new(min, max),
                Node {
                    width: px(200.0),
                    height: px(24.0),
                    padding: UiRect::all(px(2.0)),
                    border: UiRect::all(px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                BackgroundColor(SLIDER_TRACK),
                CustomUiData { name: text },
                Hovered::default(),
                observe(slider_self_update),
                children![(
                    SliderThumb,
                    Node {
                        width: px(16.0),
                        height: px(20.0),
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(0.0),
                        ..default()
                    },
                    BackgroundColor(SLIDER_THUMB),
                )],
            ),
        ],
    )
}

#[derive(Component)]
pub struct CustomUiData {
    pub(crate) name: String,
}