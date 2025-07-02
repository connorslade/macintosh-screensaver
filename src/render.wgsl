struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vert(
    @location(0) pos: vec4<f32>,
    @location(1) uv: vec2<f32>,
) -> VertexOutput {
    return VertexOutput(vec4(pos.x, -pos.y, pos.z, pos.w), uv);
}

@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read> image: array<u32>;

struct Uniform {
    pan: vec2f,

    image_size: vec2u,
    window_size: vec2u
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    let px = vec2u(in.uv * vec2f(ctx.image_size));
    let idx = px.y * ctx.image_size.x + px.x;
    let pixel = (image[idx / 32] & (1u << (idx % 32))) != 0;

    if pixel {
        return vec4(vec3(1.0), 1.0);
    } else {
        return vec4(vec3(0.0), 1.0);
    }
}
