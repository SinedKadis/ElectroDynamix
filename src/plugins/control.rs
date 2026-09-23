use crate::block::{BlockState, Blocks, SelectedState};
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::math::ops::powf;
use bevy::prelude::*;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, controls);
        app.insert_resource(SelectedState{ state: BlockState::Copper });

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
    window: Single<&Window>,
    mut blocks: ResMut<Blocks>,
    sel_state: Res<SelectedState>,
    interaction_query: Query<
        (
            &Interaction,
        )
    >,

) {

    let (camera, mut transform, mut projection,camera_transform)
        = camera_query.into_inner();

    on_mouse_click(&mouse_button_input, window, &mut blocks, &interaction_query, camera, camera_transform, sel_state);

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

fn on_mouse_click(
    mouse_button_input: &Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    blocks: &mut ResMut<Blocks>,
    interaction_query: &Query<(&Interaction,)>,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    sel_state: Res<SelectedState>,
) {
    if !mouse_button_input.pressed(MouseButton::Left) {
        return;
    }

    let is_interacting_with_ui = interaction_query
        .iter()
        .any(|(interaction,)| *interaction != Interaction::None);
    if is_interacting_with_ui {
        return;
    }

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let x = world_pos.x.floor() as i32;
    let y = world_pos.y.floor() as i32;

    if let Some(existing_block) = blocks
        .block_data
        .iter_mut()
        .find(|(bx, by, _)| *bx == x && *by == y)
    {
        existing_block.2 = sel_state.state;
    } else {
        blocks.block_data.push((x, y, sel_state.state));
    }
}
