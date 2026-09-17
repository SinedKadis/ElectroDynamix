use bevy::{input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
                prelude::*,};

pub(crate) struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, (mouse_click_system, mouse_move_system));

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
// This system prints messages when you press or release the left mouse button:
fn mouse_click_system(mouse_button_input: Res<ButtonInput<MouseButton>>, query: Query<&MousePosition, With<Mouse>>) {
    if mouse_button_input.pressed(MouseButton::Left) {
        // info!("left mouse currently pressed");
        on_mouse_pressed();
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        on_mouse_click(query);
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        info!("left mouse just released");
    }
}

fn on_mouse_click(query: Query<&MousePosition, With<Mouse>>) {
    if let Ok(mouse_position) = query.single() {
        info!("left mouse just pressed at ({}, {})", mouse_position.x, mouse_position.y);
    }
}

fn on_mouse_pressed() {

}


// This system prints messages when you finish dragging or scrolling with your mouse
fn mouse_move_system(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    accumulated_mouse_scroll: Res<AccumulatedMouseScroll>,
    mut query: Query<&mut MousePosition, With<Mouse>>
) {
    if accumulated_mouse_motion.delta != Vec2::ZERO {
        let delta = accumulated_mouse_motion.delta;
        if let Ok(mut mouse_position) = query.single_mut() {
            mouse_position.x += delta.x as i32;
            mouse_position.y += delta.y as i32;
        }
        // info!("mouse moved ({}, {})", delta.x, delta.y);
    }
    if accumulated_mouse_scroll.delta != Vec2::ZERO {
        let delta = accumulated_mouse_scroll.delta;
        // info!("mouse scrolled ({}, {})", delta.x, delta.y);
    }
}
