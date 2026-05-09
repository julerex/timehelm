#import bevy_pbr::forward_io::VertexOutput

struct ShipClipMaterial {
    clip_data: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: ShipClipMaterial;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var deck_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var deck_samp: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = select(1.0, material.clip_data.y, in.world_position.z > material.clip_data.x);
    let uv = vec2<f32>(
        in.world_position.x / 60.0 + 0.5,
        in.world_position.y / 318.0 + 0.5,
    );
    let pat = textureSample(deck_tex, deck_samp, uv).rgb;
#ifdef VERTEX_COLORS
    let base = in.color;
    let rgb = base.rgb * pat * 1.08;
    return vec4<f32>(rgb, base.a * alpha);
#else
    let rgb = pat;
    return vec4<f32>(rgb, alpha);
#endif
}
