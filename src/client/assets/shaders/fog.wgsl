// Volumetric distance fog post-process shader
// Applies depth-based fog with height falloff and sky color blending

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct FogSettings {
    fog_color: vec4<f32>,
    sky_color: vec4<f32>,
    fog_start: f32,
    fog_end: f32,
    fog_density: f32,
    height_falloff: f32,
    sun_scatter_strength: f32,
    time: f32,
};

@group(0) @binding(2) var depth_texture: texture_depth_2d;
@group(0) @binding(3) var<uniform> fog: FogSettings;

fn linear_depth(raw_depth: f32, near: f32, far: f32) -> f32 {
    return near * far / (far - raw_depth * (far - near));
}

fn exponential_fog(dist: f32, density: f32) -> f32 {
    return 1.0 - exp(-density * dist * dist);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, texture_sampler, in.uv);
    let raw_depth = textureSample(depth_texture, texture_sampler, in.uv);

    // Reconstruct linear depth (approximate, tune near/far to match camera)
    let near = 0.1;
    let far = 500.0;
    let lin_depth = linear_depth(raw_depth, near, far);

    // Exponential squared fog
    let fog_factor = exponential_fog(lin_depth, fog.fog_density * 0.001);
    let fog_factor_clamped = clamp(fog_factor, 0.0, 1.0);

    // Sky gradient based on UV.y (top = deeper sky, bottom = horizon)
    let horizon = 1.0 - clamp(in.uv.y * 2.0, 0.0, 1.0);
    let blend_color = mix(fog.fog_color, fog.sky_color, horizon);

    // Sun scattering: brighter fog near screen center at low sun angles
    // (approximate — real volumetric scattering needs ray marching)
    let center = vec2<f32>(0.5, 0.6); // horizon-ish
    let sun_dist = length(in.uv - center);
    let sun_scatter = fog.sun_scatter_strength * exp(-sun_dist * 4.0) * fog_factor_clamped;
    let sun_color = vec4<f32>(1.0, 0.85, 0.5, 0.0) * sun_scatter;

    let final_color = mix(color, blend_color, fog_factor_clamped) + sun_color;
    return vec4<f32>(final_color.rgb, 1.0);
}
