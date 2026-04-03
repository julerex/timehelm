#import bevy_pbr::forward_io::VertexOutput

struct ShipClipMaterial {
    clip_data: vec4<f32>,
}

@group(2) @binding(0) var<uniform> material: ShipClipMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    if in.world_position.z > material.clip_data.x {
        discard;
    }
#ifdef VERTEX_COLORS
    return in.color;
#else
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
#endif
}
