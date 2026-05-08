#import bevy_pbr::forward_io::VertexOutput

struct ShipClipMaterial {
    clip_data: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: ShipClipMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = select(1.0, material.clip_data.y, in.world_position.z > material.clip_data.x);
#ifdef VERTEX_COLORS
    let base = in.color;
    return vec4<f32>(base.rgb, base.a * alpha);
#else
    return vec4<f32>(1.0, 0.0, 1.0, alpha);
#endif
}
