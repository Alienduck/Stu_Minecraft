pub mod chunk;
pub mod generator;
pub mod registry;

use bevy::prelude::*;
use chunk::{Chunk, ChunkCoord};
use generator::TerrainGenerator;
use registry::BlockRegistry;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BlockRegistry::new())
            .insert_resource(TerrainGenerator::new(42))
            .add_systems(Startup, spawn_initial_chunks);
    }
}

fn spawn_initial_chunks(
    mut commands: Commands,
    generator: Res<TerrainGenerator>,
    registry: Res<BlockRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    const RENDER_DISTANCE: i32 = 4;
    for cx in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for cz in -RENDER_DISTANCE..=RENDER_DISTANCE {
            let coord = ChunkCoord { x: cx, z: cz };
            let chunk = generator.generate_chunk(coord);
            let mesh = chunk::build_chunk_mesh(&chunk, &registry);
            let mesh_handle = meshes.add(mesh);

            commands.spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    perceptual_roughness: 1.0,
                    ..default()
                })),
                Transform::from_xyz(
                    (cx * chunk::CHUNK_SIZE as i32) as f32,
                    0.0,
                    (cz * chunk::CHUNK_SIZE as i32) as f32,
                ),
                chunk,
                coord,
            ));
        }
    }
}
