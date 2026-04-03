#ifdef PREPASS_FRAGMENT
#import bevy_pbr::prepass_io::{FragmentOutput, VertexOutput}

struct ShipClipMaterial {
    clip_data: vec4<f32>,
}

@group(2) @binding(0) var<uniform> material: ShipClipMaterial;

@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    if in.world_position.z > material.clip_data.x {
        discard;
    }
    var out: FragmentOutput;
    return out;
}
#endif
