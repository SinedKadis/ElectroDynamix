use bevy::{
    camera::Viewport,
    color::palettes::
    css::GREEN,
    prelude::*,
};

pub(crate) struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(PostStartup, setup_camera);
        app.add_systems(FixedUpdate, fix_viewpoint);

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
            "Use the mouse wheel to zoom in and out.\n\
                  Use the WASD keys to move the camera.",
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
    let ( _camera, _transform, mut projection) = camera_query.into_inner();

    if let Projection::Orthographic(projection2d) = &mut *projection {
        projection2d.scale = 0.1;
    }
}

fn fix_viewpoint(camera_query: Single<(&mut Camera, &Transform, &Projection)>,
                 window: Single<&Window>,){
    let (mut camera, _transform, _projection) = camera_query.into_inner();

    let window_size = window.resolution.physical_size();
    if let Some(viewport) = camera.viewport.as_mut() {
        // Reset viewport size on window resize
        if viewport.physical_size.x != window_size.x || viewport.physical_size.y != window_size.y {
            viewport.physical_size = window_size;
        }
    }
}



