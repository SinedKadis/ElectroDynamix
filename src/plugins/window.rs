use bevy::input_focus::InputFocus;
use bevy::
prelude::*;

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFocus>();
        app.add_systems(Startup, setup);
        app.add_systems(PostStartup, setup_camera);
    }
}

pub const BACKGROUND_COLOR: Color = Color::linear_rgb(0.01, 0.01, 0.01);
pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(0.0, 0.0, 0.0);

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
            button_bundle(String::from("Resume")),
        ],
    ));
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
        ButtonData { name: text},
        Visibility::Visible
    )
}

#[derive(Component)]
pub struct ButtonData {
    pub(crate) name: String,
}