use bevy::{asset::RenderAssetUsages, prelude::*, render::render_resource::PrimitiveTopology};

use shared::{
    chunk::{CHUNK_SIZE, ChunkCoord, build_chunk_mesh},
    generator::TerrainGenerator,
};

use crate::net::{EvBlockUpdate, EvWelcome, NetSender};

pub use shared::chunk::{CHUNK_HEIGHT, Chunk};

/// Marks a chunk entity and stores its grid coordinate.
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkCoordComp(pub ChunkCoord);

#[derive(Resource)]
pub struct WorldSeed(pub u32);

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (on_welcome, apply_block_updates));
    }
}

/// Player max distance view (minecraft inspiration LOL)
const RENDER_DISTANCE: i32 = 4;

/// Spawn all chunks once we receive the seed from the server.
fn on_welcome(
    mut commands: Commands,
    mut ev: MessageReader<EvWelcome>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing: Query<Entity, With<ChunkCoordComp>>,
    sender: Res<NetSender>,
) {
    let Some(welcome) = ev.read().last() else {
        return;
    };

    // Despawn any previously-loaded chunks (reconnect case)
    for e in existing.iter() {
        commands.entity(e).despawn();
    }

    commands.insert_resource(WorldSeed(welcome.seed));

    let generator = TerrainGenerator::new(welcome.seed);
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 1.0,
        ..default()
    });

    for cx in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for cz in -RENDER_DISTANCE..=RENDER_DISTANCE {
            let coord = ChunkCoord { x: cx, z: cz };
            let chunk = generator.generate_chunk(coord);
            let raw = build_chunk_mesh(&chunk);
            let mesh = assemble_mesh(raw);

            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(
                    (cx * CHUNK_SIZE as i32) as f32,
                    0.0,
                    (cz * CHUNK_SIZE as i32) as f32,
                ),
                ChunkCoordComp(coord),
                chunk,
            ));
        }
    }
    let _ = sender
        .0
        .lock()
        .unwrap()
        .send(shared::protocol::ClientPacket::Ready);
}

/// Apply a server-authoritative block update to the matching chunk entity.
fn apply_block_updates(
    mut ev: MessageReader<EvBlockUpdate>,
    mut chunks: Query<(&mut Chunk, &ChunkCoordComp, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for update in ev.read() {
        for (mut chunk, coord_comp, mesh3d) in chunks.iter_mut() {
            if coord_comp.0 != update.coord {
                continue;
            }

            chunk.set(
                update.lx as usize,
                update.ly as usize,
                update.lz as usize,
                update.block,
            );

            let raw = build_chunk_mesh(&chunk);
            let new = assemble_mesh(raw);
            if let Some(m) = meshes.get_mut(&mesh3d.0) {
                *m = new;
            }
            break;
        }
    }
}

pub fn assemble_mesh(raw: shared::chunk::RawMeshData) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, raw.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, raw.normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, raw.uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, raw.colors);
    mesh.insert_indices(bevy::mesh::Indices::U32(raw.indices));
    mesh
}
