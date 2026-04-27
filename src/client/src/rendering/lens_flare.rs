// src/client/src/rendering/lens_flare.rs

use bevy::{
    light::NotShadowCaster,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};

pub struct LensFlarePlugin;

impl Plugin for LensFlarePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<LensFlareMaterial>::default())
            .add_systems(Startup, spawn_lens_flare)
            .add_systems(Update, (update_flare_billboard, fade_flare_by_sun_angle));
    }
}

// ── Uniform data ─────────────────────────────────────────────────────────────

/// Must exactly mirror `LensFlareUniform` in lens_flare.wgsl.
#[derive(Clone, ShaderType)]
pub struct LensFlareUniform {
    pub core_color: Vec4,
    pub halo_color: Vec4,
    pub streak_color: Vec4,
    /// x=intensity, y=core_radius, z=halo_radius, w=streak_length
    pub params0: Vec4,
    /// x=streak_width, y=ghost_intensity, z=ghost_spacing, w=num_ghosts (f32)
    pub params1: Vec4,
}

impl From<&LensFlareMaterial> for LensFlareUniform {
    fn from(m: &LensFlareMaterial) -> Self {
        m.uniform.clone()
    }
}

// ── Material ──────────────────────────────────────────────────────────────────

#[derive(Asset, TypePath, AsBindGroup, Clone)]
#[uniform(0, LensFlareUniform)]
pub struct LensFlareMaterial {
    pub uniform: LensFlareUniform,
}

impl Material for LensFlareMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/lens_flare.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }
}

impl Default for LensFlareMaterial {
    fn default() -> Self {
        Self {
            uniform: LensFlareUniform {
                core_color: Vec4::new(1.0, 0.97, 0.8, 1.0),
                halo_color: Vec4::new(0.8, 0.85, 1.0, 1.0),
                streak_color: Vec4::new(1.0, 0.90, 0.5, 1.0),
                params0: Vec4::new(1.8, 0.08, 0.35, 0.9),
                params1: Vec4::new(0.012, 0.5, 0.25, 4.0),
            },
        }
    }
}

// ── Components ────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct LensFlare;

// ── Systems ───────────────────────────────────────────────────────────────────

fn spawn_lens_flare(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut flare_mats: ResMut<Assets<LensFlareMaterial>>,
) {
    let quad = meshes.add(Rectangle::new(120.0, 120.0));
    let mat = flare_mats.add(LensFlareMaterial::default());
    commands.spawn((
        Mesh3d(quad),
        MeshMaterial3d(mat),
        Transform::from_xyz(0.0, 0.0, 0.0),
        NotShadowCaster,
        LensFlare,
    ));
}

fn update_flare_billboard(
    camera_q: Query<&GlobalTransform, With<Camera3d>>,
    sun_q: Query<&GlobalTransform, (With<crate::rendering::Sun>, With<Mesh3d>)>,
    mut flare_q: Query<&mut Transform, With<LensFlare>>,
) {
    let Ok(cam_gt) = camera_q.single() else {
        return;
    };
    let Ok(sun_gt) = sun_q.single() else { return };
    let Ok(mut flare_t) = flare_q.single_mut() else {
        return;
    };

    let sun_pos = sun_gt.translation();
    flare_t.translation = sun_pos;

    let to_cam = (cam_gt.translation() - sun_pos).normalize_or_zero();
    if to_cam.length_squared() > 0.001 {
        flare_t.rotation = Quat::from_rotation_arc(Vec3::Z, to_cam);
    }
}

fn fade_flare_by_sun_angle(
    sun_q: Query<&GlobalTransform, (With<crate::rendering::Sun>, With<Mesh3d>)>,
    camera_q: Query<&GlobalTransform, With<Camera3d>>,
    mut flare_q: Query<&mut Visibility, With<LensFlare>>,
) {
    let Ok(sun_gt) = sun_q.single() else { return };
    let Ok(cam_gt) = camera_q.single() else {
        return;
    };
    let Ok(mut vis) = flare_q.single_mut() else {
        return;
    };

    let sun_y = sun_gt.translation().y;
    let to_sun = (sun_gt.translation() - cam_gt.translation()).normalize_or_zero();
    let dot = to_sun.dot(*cam_gt.forward());

    *vis = if sun_y > 0.0 && dot > 0.2 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}
