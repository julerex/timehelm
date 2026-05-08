#ifdef PREPASS_FRAGMENT
#import bevy_pbr::prepass_io::{FragmentOutput, VertexOutput}

struct ShipClipMaterial {
    clip_data: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: ShipClipMaterial;

@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    let _alpha = select(1.0, material.clip_data.y, in.world_position.z > material.clip_data.x);
    var out: FragmentOutput;
    return out;
}
#endif
