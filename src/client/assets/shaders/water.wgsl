#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::globals
#import bevy_pbr::mesh_view_bindings::view

struct WaterMaterialUniform {
    shallow_color:   vec4<f32>,
    deep_color:      vec4<f32>,
    foam_color:      vec4<f32>,
    // x=time_scale, y=wave_amplitude, z=wave_frequency, w=fresnel_strength
    params0:         vec4<f32>,
    // x=foam_threshold, y=transparency
    params1:         vec4<f32>,
}

#ifdef BINDLESS
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<storage> material_indices: array<MaterialBindings>;
@group(#{MATERIAL_BIND_GROUP}) @binding(10) var<storage> materials: binding_array<WaterMaterialUniform>;

struct MaterialBindings { material: u32, }

fn get_material(in: VertexOutput) -> WaterMaterialUniform {
    let slot = bevy_pbr::mesh_bindings::mesh[in.instance_index].material_and_lightmap_bind_group_slot & 0xffffu;
    return materials[material_indices[slot].material];
}
#else
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: WaterMaterialUniform;

fn get_material(in: VertexOutput) -> WaterMaterialUniform {
    return material;
}
#endif

fn hash2(p: vec2<f32>) -> f32 {
    let q = vec2<f32>(dot(p, vec2<f32>(127.1, 311.7)), dot(p, vec2<f32>(269.5, 183.3)));
    return fract(sin(q.x + q.y) * 43758.5453123);
}

fn noise2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash2(i), hash2(i + vec2(1.,0.)), u.x),
        mix(hash2(i + vec2(0.,1.)), hash2(i + vec2(1.,1.)), u.x),
        u.y
    );
}

fn fbm(p: vec2<f32>, time: f32) -> f32 {
    var val = 0.0; var amp = 0.5; var freq = 1.0; var pp = p;
    for (var i = 0; i < 4; i++) {
        let fi = f32(i) + 1.0;
        val  += noise2d(pp * freq + vec2(time * 0.3 * fi, time * 0.2 * fi)) * amp;
        amp  *= 0.5; freq *= 2.1; pp += vec2(val * 0.1);
    }
    return val;
}

fn water_normal(uv: vec2<f32>, time: f32, amp: f32) -> vec3<f32> {
    let e = 0.02;
    let h  = fbm(uv, time);
    let dx = (fbm(uv + vec2(e, 0.), time) - h) / e * amp;
    let dz = (fbm(uv + vec2(0., e), time) - h) / e * amp;
    return normalize(vec3(-dx, 1.0, -dz));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let mat = get_material(in);

    let time_scale       = mat.params0.x;
    let wave_amplitude   = mat.params0.y;
    let wave_frequency   = mat.params0.z;
    let fresnel_strength = mat.params0.w;
    let foam_threshold   = mat.params1.x;
    let transparency     = mat.params1.y;

    let time     = globals.time * time_scale;
    let uv       = in.uv * wave_frequency;
    let normal   = water_normal(uv, time, wave_amplitude);
    let view_dir = normalize(view.world_position - in.world_position.xyz);
    let fresnel  = pow(1.0 - max(dot(normal, view_dir), 0.0), fresnel_strength);

    let h = fbm(uv * 0.5, time * 0.5);
    var color = mix(mat.shallow_color, mat.deep_color, smoothstep(0.3, 0.7, h));
    color = mix(color, mat.foam_color, smoothstep(foam_threshold, 1.0, h) * 0.6);
    color = mix(color, vec4(0.5, 0.72, 1.0, 1.0), fresnel * 0.5);

    let sun_dir = normalize(vec3(0.6, 0.8, 0.3));
    let spec    = pow(max(dot(reflect(-sun_dir, normal), view_dir), 0.0), 64.0);
    color += vec4(1.0, 0.98, 0.8, 0.0) * spec * 0.8;

    return vec4(color.rgb, mix(transparency, 0.95, fresnel));
}
