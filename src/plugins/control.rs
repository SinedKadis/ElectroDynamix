use crate::block::Blocks;
use bevy::color::palettes::css::BLUE;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::math::ops::powf;
use bevy::prelude::*;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, controls);

    }
}

// fn setup(mut commands: Commands) {
//
// }



const MOUSE_SENSITIVITY: f32 = 1f32;

fn controls(
    camera_query: Single<(&Camera, &mut Transform, &mut Projection,&GlobalTransform)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Fixed>>,
    accumulated_mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
    mut blocks: ResMut<Blocks>
) {

    let (camera, mut transform, mut projection,camera_transform)
        = camera_query.into_inner();

    if mouse_button_input.pressed(MouseButton::Left) {
        // info!("left mouse currently pressed");
        if let Some(cursor_position) = window.cursor_position()
            && let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position)
        {

            let x = &(world_pos.x.floor() as i32);
            let y = &(world_pos.y.floor() as i32);

            if blocks.pos.contains(&(*x, *y)) {
                return;
            }


            blocks.pos.push((*x, *y));
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
                MeshMaterial2d(materials.add(Color::from(BLUE))),
                Transform::from_xyz(*x as f32 + 0.5,
                                    *y as f32 + 0.5, 1.0)
            ));
        }

    }

    if mouse_button_input.just_pressed(MouseButton::Left) {

    }

    if mouse_button_input.just_released(MouseButton::Left) {

    }
    





    // Camera zoom controls
    if let Projection::Orthographic(projection2d) = &mut *projection {
        if accumulated_mouse_scroll.delta != Vec2::ZERO {
            let delta = accumulated_mouse_scroll.delta;
            if delta.y < 0f32 {
                projection2d.scale *= powf(4.0f32, time.delta_secs() * MOUSE_SENSITIVITY * 2f32);
            } else {
                projection2d.scale *= powf(0.25f32, time.delta_secs() * MOUSE_SENSITIVITY * 2f32);
            }
        }

        let fspeed = 600.0 * time.delta_secs() * projection2d.scale;
        // Camera movement controls
        if input.pressed(KeyCode::KeyW) {
            transform.translation.y += fspeed;
        }
        if input.pressed(KeyCode::KeyS) {
            transform.translation.y -= fspeed;
        }
        if input.pressed(KeyCode::KeyA) {
            transform.translation.x -= fspeed;
        }
        if input.pressed(KeyCode::KeyD) {
            transform.translation.x += fspeed;
        }
    }
}
