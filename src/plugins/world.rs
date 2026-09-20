use crate::block::Blocks;
use crate::plugins::config::Config;
use bevy::color::palettes::basic::WHITE;
use bevy::color::palettes::css::BLUE;
use bevy::prelude::*;
use bevy_vector_shapes::Shape2dPlugin;
use bevy_vector_shapes::painter::ShapePainter;
use bevy_vector_shapes::shapes::RectPainter;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, draw_grid.after(TransformSystems::Propagate));
        app.add_plugins(Shape2dPlugin::default());
        app.insert_resource(Blocks{ pos: Vec::new()});
        app.add_systems(Update, draw_blocks);
    }
}

const GRID_COLOR: Srgba = WHITE;

fn draw_grid(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut gizmos: Gizmos,
    config: Res<Config>
) {
    if !config.draw_grid { return; }
    let (camera, camera_transform) = *camera_query;

    if let Some(cursor_position) = window.cursor_position()
        && let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position)
    {
        gizmos.grid_2d(world_pos.round(), UVec2::new(10, 10), Vec2::new(1., 1.), GRID_COLOR);
    }
}
fn draw_blocks(mut painter: ShapePainter, blocks: Res<Blocks>) {
    for pos in &blocks.pos {
        painter.color = Color::from(BLUE);
        painter.translate(Vec3::new(pos.0 as f32 + 0.5, pos.1 as f32 + 0.5, 1.0));

        painter.rect(Vec2::splat(1.0));

        painter.reset();
    }
}