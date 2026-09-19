use bevy::{
    camera::Viewport,
    color::palettes::
    css::GREEN,
    prelude::*,
};
use bevy::math::ops::powf;

pub(crate) struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(PostStartup, setup_camera);

    }
}

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
            viewport: Some(Viewport {
                physical_position: (Vec2::ZERO).as_uvec2(),
                physical_size: window.resolution.physical_size(),
                ..default()
            }),
            ..default()
        },
    ));

    // Create a minimal UI explaining how to interact with the example
    commands.spawn((
        Text::new(
            "Move the mouse to see the circle follow your cursor.\n\
                    Use the arrow keys to move the camera.\n\
                    Use the comma and period keys to zoom in and out.\n\
                    Use the WASD keys to move the viewport.\n\
                    Use the IJKL keys to resize the viewport.",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));

    // Add mesh to make camera movement visible
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(40.0, 20.0))),
        MeshMaterial2d(materials.add(Color::from(GREEN))),
    ));

    // Add background to visualize viewport bounds
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(50000.0, 50000.0))),
        MeshMaterial2d(materials.add(Color::linear_rgb(0.01, 0.01, 0.01))),
        Transform::from_translation(Vec3::new(0.0, 0.0, -200.0)),
    ));


}

fn setup_camera(camera_query: Single<(&mut Camera, &mut Transform, &mut Projection)>){
    let ( _camera, _transform, projection) = camera_query.into_inner();

    if let Projection::Orthographic(projection2d) = & *projection {
        // if accumulated_mouse_scroll.delta != Vec2::ZERO {
        //     let delta = accumulated_mouse_scroll.delta;
        //     if delta.y < 0f32 {
        //         projection2d.scale *= powf(4.0f32, time.delta_secs() * crate::control::MOUSE_SENSITIVITY * 2f32);
        //     } else {
        //         projection2d.scale *= powf(0.25f32, time.delta_secs() * crate::control::MOUSE_SENSITIVITY * 2f32);
        //     }
        // }
        info!("Camera scale: {}", projection2d.scale);
    }
}



