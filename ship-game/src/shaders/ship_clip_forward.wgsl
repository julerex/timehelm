#import bevy_pbr::forward_io::VertexOutput

// Daylight for decks and bulkheads. `sun.xyz` points at the directional light,
// `eye.xyz` is the camera. Vertical faces get a tight specular lobe; deck tops stay matte.
struct ShipClipMaterial {
    clip_data: vec4<f32>,
    sun: vec4<f32>,
    eye: vec4<f32>,
    // Eight lamps. xyz is the position in metres, w is the radius. w <= 0 is unused.
    // Same count as LAMP_LIMIT in lamps.rs.
    lamps: array<vec4<f32>, 8>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: ShipClipMaterial;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var deck_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var deck_samp: sampler;

fn safe_normalize(v: vec3<f32>) -> vec3<f32> {
    let len2 = dot(v, v);
    if (len2 < 1e-8) {
        return vec3<f32>(0.0, 0.0, 1.0);
    }
    return v * inverseSqrt(len2);
}

// Same cubic as `one_lamp` in lamps.rs. 1 at the lamp, under 0.05 at 20 m for a 6 m radius.
fn lamp_gain(world_pos: vec3<f32>) -> f32 {
    var gain = 0.0;
    for (var i = 0u; i < 8u; i = i + 1u) {
        let lamp = material.lamps[i];
        if (lamp.w > 0.0) {
            let q = length(world_pos - lamp.xyz) / lamp.w;
            gain = gain + 1.0 / (1.0 + q * q * q);
        }
    }
    return gain;
}

fn daylight(albedo: vec3<f32>, world_pos: vec3<f32>, world_normal: vec3<f32>) -> vec3<f32> {
    let n = safe_normalize(world_normal);
    let to_sun = safe_normalize(material.sun.xyz);
    let ndotl = clamp(dot(n, to_sun), 0.0, 1.0);
    // Sky fill from above, a little sea bounce from below.
    let sky = mix(
        vec3<f32>(0.12, 0.11, 0.10),
        vec3<f32>(0.38, 0.44, 0.52),
        clamp(n.z * 0.5 + 0.5, 0.0, 1.0),
    );
    let sun_rgb = vec3<f32>(1.0, 0.96, 0.88) * material.sun.w;
    var color = albedo * (sky + sun_rgb * ndotl);

    let to_eye = material.eye.xyz - world_pos;
    let view = safe_normalize(to_eye);
    let half_dir = safe_normalize(to_sun + view);
    // Painted steel on walls and hull skirts. Floor coverings stay diffuse.
    let vertical = 1.0 - abs(n.z);
    let gloss = pow(clamp(dot(n, half_dir), 0.0, 1.0), mix(16.0, 72.0, vertical));
    color = color + sun_rgb * gloss * vertical * material.eye.w;
    color = color + albedo * vec3<f32>(1.0, 0.78, 0.45) * lamp_gain(world_pos);
    return color;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = select(1.0, material.clip_data.y, in.world_position.z > material.clip_data.x);
    let uv = vec2<f32>(
        in.world_position.x / 60.0 + 0.5,
        in.world_position.y / 318.0 + 0.5,
    );
    let pat = textureSample(deck_tex, deck_samp, uv).rgb;
#ifdef VERTEX_COLORS
    let albedo = in.color.rgb * pat;
    let opacity = in.color.a * alpha;
#else
    let albedo = pat;
    let opacity = alpha;
#endif
    return vec4<f32>(daylight(albedo, in.world_position.xyz, in.world_normal), opacity);
}
