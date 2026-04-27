#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::globals

struct LensFlareUniform {
    core_color:   vec4<f32>,
    halo_color:   vec4<f32>,
    streak_color: vec4<f32>,
    // x=intensity, y=core_radius, z=halo_radius, w=streak_length
    params0:      vec4<f32>,
    // x=streak_width, y=ghost_intensity, z=ghost_spacing, w=num_ghosts (f32)
    params1:      vec4<f32>,
}

#ifdef BINDLESS
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<storage> material_indices: array<MaterialBindings>;
@group(#{MATERIAL_BIND_GROUP}) @binding(10) var<storage> materials: binding_array<LensFlareUniform>;

struct MaterialBindings { material: u32, }

fn get_material(in: VertexOutput) -> LensFlareUniform {
    let slot = bevy_pbr::mesh_bindings::mesh[in.instance_index].material_and_lightmap_bind_group_slot & 0xffffu;
    return materials[material_indices[slot].material];
}
#else
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: LensFlareUniform;

fn get_material(in: VertexOutput) -> LensFlareUniform {
    return material;
}
#endif

fn hex_ghost(uv: vec2<f32>, center: vec2<f32>, size: f32) -> f32 {
    return smoothstep(1.0, 0.4, length(uv - center) / max(size, 0.001));
}

fn streak(uv: vec2<f32>, width: f32, len_scale: f32) -> f32 {
    return exp(-abs(uv.y) / max(width, 0.001))
         * (1.0 - exp(-abs(uv.x) / max(len_scale, 0.001)) * 0.3);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let mat = get_material(in);

    let intensity       = mat.params0.x;
    let core_radius     = mat.params0.y;
    let halo_radius     = mat.params0.z;
    let streak_length   = mat.params0.w;
    let streak_width    = mat.params1.x;
    let ghost_intensity = mat.params1.y;
    let ghost_spacing   = mat.params1.z;
    let num_ghosts      = i32(mat.params1.w);

    let uv   = (in.uv - 0.5) * 2.0;
    let dist = length(uv);

    let core    = smoothstep(core_radius, 0.0, dist) * intensity;
    let halo    = smoothstep(halo_radius, core_radius * 0.5, dist) * intensity * 0.4;
    let streaks = (streak(uv, streak_width, streak_length)
                 + streak(uv.yx, streak_width * 0.5, streak_length * 0.3) * 0.4)
                 * intensity * 0.6;

    var ghost_total = 0.0;
    for (var i = 0; i < num_ghosts; i++) {
        let fi = f32(i);
        ghost_total += hex_ghost(uv, -uv * ghost_spacing * (fi + 1.0), 0.08 + fi * 0.05)
                     * (0.5 + 0.5 * sin(fi * 1.4));
    }

    var color = mat.core_color * core + mat.halo_color * halo + mat.streak_color * streaks;
    color += vec4(
        0.8 + 0.2 * sin(ghost_total * 3.0),
        0.7 + 0.3 * cos(ghost_total * 2.0),
        1.0, 1.0
    ) * ghost_total * ghost_intensity * 0.3;

    color *= 1.0 + 0.04 * sin(globals.time * 2.5);
    let alpha = clamp(color.r + color.g + color.b, 0.0, 1.0) * 0.85;
    return vec4(color.rgb, alpha);
}
