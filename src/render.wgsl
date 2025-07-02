@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read> image: array<u32>;

struct Uniform {
    view: mat4x4f,
    image_size: vec2u,
    window_size: vec2u
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vert(
    @location(0) pos: vec4<f32>,
    @location(1) uv: vec2<f32>,
) -> VertexOutput {
    return VertexOutput(ctx.view * pos, uv);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    let px = vec2u(in.uv * vec2f(ctx.image_size));
    let idx = px.y * ctx.image_size.x + px.x;
    let pixel = (image[idx / 32] & (1u << (idx % 32))) != 0;

    let color = mix(vec3(.110, .620, .455), vec3(.325, .800, .631), f32(pixel));
    return vec4(color, 1.0);
}
