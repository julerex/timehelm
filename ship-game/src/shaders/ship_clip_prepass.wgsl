#ifdef PREPASS_FRAGMENT
#import bevy_pbr::prepass_io::{FragmentOutput, VertexOutput}

struct ShipClipMaterial {
    clip_data: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: ShipClipMaterial;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var deck_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var deck_samp: sampler;

@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    let _alpha = select(1.0, material.clip_data.y, in.world_position.z > material.clip_data.x);
    let _pat = textureSample(deck_tex, deck_samp, vec2<f32>(0.5, 0.5)).rgb;
    var out: FragmentOutput;
    return out;
}
#endif
