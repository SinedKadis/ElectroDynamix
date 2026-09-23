use crate::block::{BlockState, Blocks, Direction};
use crate::plugins::config::Config;
use bevy::color::palettes::basic::WHITE;
use bevy::color::palettes::css::ORANGE_RED;
use bevy::color::palettes::tailwind::CYAN_700;
use bevy::prelude::*;
use bevy_vector_shapes::Shape2dPlugin;
use bevy_vector_shapes::painter::ShapePainter;
use bevy_vector_shapes::shapes::RectPainter;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, draw_grid.after(TransformSystems::Propagate));
        app.add_plugins(Shape2dPlugin::default());
        app.insert_resource(Blocks{ block_data: Vec::new()});
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
fn draw_blocks(mut painter: ShapePainter, blocks: Res<Blocks>, mut gizmos: Gizmos) {
    for block_data in &blocks.block_data {
        let visual_y = block_data.1 as f32 + 0.5;
        let visual_x = block_data.0 as f32 + 0.5;
        painter.color = match block_data.2 {
            BlockState::Copper => {Color::from(ORANGE_RED)}
            BlockState::Electricity(dir) => {
                gizmos.arrow_2d(match dir {
                    Direction::Up => {Vec2::new(visual_x, visual_y-0.5)}
                    Direction::Down => {Vec2::new(visual_x,visual_y+0.5)}
                    Direction::Left => {Vec2::new(visual_x+0.5, visual_y)}
                    Direction::Right => {Vec2::new(visual_x-0.5, visual_y)}
                }, match dir {
                    Direction::Up => {Vec2::new(visual_x, visual_y + 0.5)}
                    Direction::Down => {Vec2::new(visual_x, visual_y - 0.5)}
                    Direction::Left => {Vec2::new(visual_x - 0.5, visual_y)}
                    Direction::Right => {Vec2::new(visual_x + 0.5, visual_y)}
                }, WHITE);
                Color::from(CYAN_700)
            }
        };
        painter.translate(Vec3::new(visual_x, visual_y, 1.0));

        painter.rect(Vec2::splat(1.0));

        painter.reset();
    }
}