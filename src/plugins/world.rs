use bevy::color::palettes::basic::WHITE;
use bevy::prelude::*;
use crate::block::Blocks;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, draw_grid.after(TransformSystems::Propagate));
            app.insert_resource(Blocks{ pos: Vec::new()});
    }
}

const GRID_COLOR: Srgba = WHITE;

fn draw_grid(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = *camera_query;

    if let Some(cursor_position) = window.cursor_position()
        && let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position)
    {
        gizmos.grid_2d(world_pos.round(), UVec2::new(10, 10), Vec2::new(1., 1.), GRID_COLOR);
        // gizmos.circle_2d(world_pos, 10., WHITE);
        // // Should be the same as world_pos
        // gizmos.circle_2d(world_check, 8., RED);
    }
}