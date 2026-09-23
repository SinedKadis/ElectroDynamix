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
    if mouse_button_input.pressed(MouseButton::Left) {
        // info!("left mouse currently pressed");
        for interaction in interaction_query {
            if interaction != (&Interaction::None,) {
                return;
            }
        }
        if let Some(cursor_position) = window.cursor_position()
            && let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position)
        {
            let x = &(world_pos.x.floor() as i32);
            let y = &(world_pos.y.floor() as i32);

            if blocks.block_data.contains(&(*x, *y, sel_state.state)) {
                return;
            }


            blocks.block_data.push((*x, *y, sel_state.state));
            // commands.spawn((
            //     Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
            //     MeshMaterial2d(materials.add(Color::from(BLUE))),
            //     Transform::from_xyz(*x as f32 + 0.5,
            //                         *y as f32 + 0.5, 1.0)
            // ));
        }
    }
}
