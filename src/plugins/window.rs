use crate::plugins::config::Config;
use bevy::color::palettes::css::RED;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::{
    color::palettes::
    css::GREEN,
    prelude::*,
};

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFocus>();
        app.add_systems(Startup, setup);
        app.add_systems(PostStartup, setup_camera);
        app.add_systems(Update,button_system);
    }
}

const BACKGROUND_COLOR: Color = Color::linear_rgb(0.01, 0.01, 0.01);
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.0,0.0,0.0);
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {

    // Initialize centered, non-window-filling viewport
    commands.spawn((
        Camera2d,
        Camera {
            ..default()
        },
    ));

    // Create a minimal UI explaining how to interact with the example
    // commands.spawn((
    //     Text::new(
    //         "Use the mouse wheel to zoom in and out.\n\
    //               Use the WASD keys to move the camera.",
    //     ),
    //     Node {
    //         position_type: PositionType::Absolute,
    //         top: px(12),
    //         left: px(12),
    //         ..default()
    //     },
    // ));

    // Add mesh to make camera movement visible
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(40.0, 20.0))),
        MeshMaterial2d(materials.add(Color::from(GREEN))),
    ));

    // Add background to visualize viewport bounds
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(50000.0, 50000.0))),
        MeshMaterial2d(materials.add(BACKGROUND_COLOR)),
        Transform::from_translation(Vec3::new(0.0, 0.0, -200.0)),
    ));


    commands.spawn(
        button(String::from("Toggle grid"))
    );
}

fn setup_camera(camera_query: Single<(&mut Camera, &mut Transform, &mut Projection)>){
    let ( _camera, _transform, mut projection) = camera_query.into_inner();

    if let Projection::Orthographic(projection2d) = &mut *projection {
        projection2d.scale = 0.1;
    }
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
            &Name
        ),
        Changed<Interaction>,
    >,
    mut config : ResMut<Config>,
) {
    for (entity,
        interaction,
        mut color,
        mut border_color,
        mut button,
        _children,
        name) in &mut interaction_query
    {

        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity, FocusCause::Pressed);
                *color = PRESSED_BUTTON.into();
                *border_color = BorderColor::all(RED);


                button.set_changed();
                if name.name == "Toggle grid" {
                    config.draw_grid = !config.draw_grid;
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

fn button(text : String) -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: percent(100),
            top: vh(10.0),
            left: vw(1.0),
            align_items: AlignItems::DEFAULT,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        children![(
            Button,
            Node {
                width: vmin(24),
                height: vmin(6),
                border: UiRect::all(px(5)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BorderColor::all(Color::WHITE),
            BackgroundColor(Color::BLACK),
            children![(
                Text::new(&text),
                TextFont {
                    font_size: FontSize::VMin(3.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextShadow::default(),

            )],
            Name{name : text}
        )],
    )
}

#[derive(Component)]
pub struct Name {name : String}



