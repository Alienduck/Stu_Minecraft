// src/client/src/rendering/water_material.rs

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{AsBindGroup, PrimitiveTopology, ShaderType},
    shader::ShaderRef,
};

pub struct WaterMaterialPlugin;

impl Plugin for WaterMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WaterMaterial>::default())
            .add_systems(Update, rebuild_water_on_block_update);
    }
}

// ── Uniform data ─────────────────────────────────────────────────────────────

/// Must exactly mirror `WaterMaterialUniform` in water.wgsl.
#[derive(Clone, ShaderType)]
pub struct WaterMaterialUniform {
    pub shallow_color: Vec4,
    pub deep_color: Vec4,
    pub foam_color: Vec4,
    /// x=time_scale, y=wave_amplitude, z=wave_frequency, w=fresnel_strength
    pub params0: Vec4,
    /// x=foam_threshold, y=transparency, zw=pad
    pub params1: Vec4,
}

impl From<&WaterMaterial> for WaterMaterialUniform {
    fn from(m: &WaterMaterial) -> Self {
        m.uniform.clone()
    }
}

// ── Material ──────────────────────────────────────────────────────────────────

/// Struct-level #[uniform] syntax: converts WaterMaterial → WaterMaterialUniform
/// via the From impl above, and places it at binding_array(10) for bindless mode.
/// In non-bindless mode this becomes a plain uniform at binding 0.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
#[uniform(0, WaterMaterialUniform)]
pub struct WaterMaterial {
    pub uniform: WaterMaterialUniform,
}

impl Material for WaterMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/water.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

impl Default for WaterMaterial {
    fn default() -> Self {
        Self {
            uniform: WaterMaterialUniform {
                shallow_color: Vec4::new(0.12, 0.55, 0.90, 0.85),
                deep_color: Vec4::new(0.02, 0.08, 0.45, 0.95),
                foam_color: Vec4::new(0.88, 0.94, 1.00, 1.00),
                params0: Vec4::new(0.6, 0.06, 0.35, 3.5),
                params1: Vec4::new(0.72, 0.75, 0.0, 0.0),
            },
        }
    }
}

// ── Water mesh marker ─────────────────────────────────────────────────────────

#[derive(Component)]
pub struct ChunkWaterMesh {
    pub chunk_coord: shared::chunk::ChunkCoord,
}

// ── Rebuild on block update ───────────────────────────────────────────────────

use crate::{
    net::EvBlockUpdate,
    world::{CHUNK_HEIGHT, Chunk, ChunkCoordComp},
};
use shared::{
    block::BlockType,
    chunk::{CHUNK_SIZE, ChunkCoord},
};

fn rebuild_water_on_block_update(
    mut ev: MessageReader<EvBlockUpdate>,
    chunks: Query<(&Chunk, &ChunkCoordComp, &Transform)>,
    existing_water: Query<(Entity, &ChunkWaterMesh)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut water_mats: ResMut<Assets<WaterMaterial>>,
) {
    let mut dirty: Vec<ChunkCoord> = ev.read().map(|u| u.coord).collect();
    dirty.sort_unstable_by_key(|c| (c.x, c.z));
    dirty.dedup();
    if dirty.is_empty() {
        return;
    }

    let mat = water_mats.add(WaterMaterial::default());

    for coord in dirty {
        for (entity, wm) in existing_water.iter() {
            if wm.chunk_coord == coord {
                commands.entity(entity).despawn();
                break;
            }
        }
        let Some((chunk, _, chunk_t)) = chunks.iter().find(|(_, cc, _)| cc.0 == coord) else {
            continue;
        };
        if let Some(mesh) = build_water_mesh(chunk) {
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(mat.clone()),
                Transform::from_translation(chunk_t.translation),
                ChunkWaterMesh { chunk_coord: coord },
            ));
        }
    }
}

fn build_water_mesh(chunk: &Chunk) -> Option<Mesh> {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for y in 0..CHUNK_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if chunk.get(x, y, z) != BlockType::Water {
                    continue;
                }
                let above_air = y + 1 >= CHUNK_HEIGHT || chunk.get(x, y + 1, z) == BlockType::Air;
                if !above_air {
                    continue;
                }

                let (fx, fy, fz) = (x as f32, y as f32 + 0.92, z as f32);
                let base = positions.len() as u32;
                positions.extend_from_slice(&[
                    [fx, fy, fz],
                    [fx + 1.0, fy, fz],
                    [fx + 1.0, fy, fz + 1.0],
                    [fx, fy, fz + 1.0],
                ]);
                let up = [0.0_f32, 1.0, 0.0];
                normals.extend_from_slice(&[up, up, up, up]);
                // UV carries world XZ for seamless tiling across chunks
                uvs.extend_from_slice(&[
                    [fx, fz],
                    [fx + 1.0, fz],
                    [fx + 1.0, fz + 1.0],
                    [fx, fz + 1.0],
                ]);
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
        }
    }
    if positions.is_empty() {
        return None;
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    Some(mesh)
}
