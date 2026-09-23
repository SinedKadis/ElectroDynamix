use crate::block::{BlockState, Direction, SelectedState};
use crate::plugins::config::Config;
use bevy::color::palettes::css::RED;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::
prelude::*;

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFocus>();
        app.add_systems(Startup, setup);
        app.add_systems(PostStartup, setup_camera);
        app.add_systems(Update, button_system);
    }
}

const BACKGROUND_COLOR: Color = Color::linear_rgb(0.01, 0.01, 0.01);
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.0, 0.0, 0.0);

fn setup(mut commands: Commands) {
    // 1. Camera
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(BACKGROUND_COLOR),
            ..default()
        },
    ));

    // 2. Toolbar anchored to bottom-left in a horizontal row
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
        Name { name: text },
    )
}

fn button_system(
    mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Button,
            &Children,
            &Name,
        ),
        Changed<Interaction>,
    >,
    mut config: ResMut<Config>,
    mut selection: ResMut<SelectedState>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, _children, name) in
        &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity, FocusCause::Pressed);
                *color = PRESSED_BUTTON.into();
                *border_color = BorderColor::all(RED);

                button.set_changed();
                if name.name == "Toggle Grid" {
                    config.draw_grid = !config.draw_grid;
                }else if name.name == "Toggle Arrows" {
                    config.draw_arrows = !config.draw_arrows;
                } else if name.name == "Copper" {
                    selection.state = BlockState::Copper;
                } else if name.name == "Electricity Right" {
                    selection.state = BlockState::Electricity(Direction::Right);
                } else if name.name == "Electricity Left" {
                    selection.state = BlockState::Electricity(Direction::Left);
                } else if name.name == "Electricity Up" {
                    selection.state = BlockState::Electricity(Direction::Up);
                } else if name.name == "Electricity Down" {
                    selection.state = BlockState::Electricity(Direction::Down);
                }
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

#[derive(Component)]
pub struct Name {
    name: String,
}