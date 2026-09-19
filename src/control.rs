use bevy::{prelude::*,};
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::math::ops::powf;

pub(crate) struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, controls);

    }
}

fn setup(mut commands: Commands) {
    commands.spawn((Mouse,MousePosition{x:0,y:0}));
}

#[derive(Component)]
struct Mouse;

#[derive(Component)]
struct MousePosition {
    x: i32,
    y: i32
}


fn on_mouse_click(query: Query<&MousePosition, With<Mouse>>) {
    if let Ok(mouse_position) = query.single() {
        info!("left mouse just pressed at ({}, {})", mouse_position.x, mouse_position.y);
    }
}

fn on_mouse_pressed() {

}

const MOUSE_SENSITIVITY: f32 = 1f32;

fn controls(
    camera_query: Single<(&Camera, &mut Transform, &mut Projection)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Fixed>>,
    accumulated_mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    query: Query<&MousePosition, With<Mouse>>
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        // info!("left mouse currently pressed");
        on_mouse_pressed();
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        on_mouse_click(query);
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        //info!("left mouse just released");
    }
    
    let (_camera, mut transform, mut projection) = camera_query.into_inner();

    
    
    
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
