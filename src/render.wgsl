@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read> image: array<u32>;

struct Uniform {
    view: mat4x4f,
    image_size: vec2u,
    window_size: vec2u,
    cutoff: f32,
    progress: f32
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
    let pos = in.uv * vec2f(ctx.image_size) - vec2(0.5);
    let rounded = round(pos);

    let vec = pos - rounded;
    let dist = max(abs(vec.x), abs(vec.y));

    let px = vec2u(rounded);
    let idx = px.y * ctx.image_size.x + px.x;
    let pixel = (image[idx / 32] & (1u << (idx % 32))) != 0 || dist > (ctx.cutoff * saturate(in.uv.x * 20.0 + ctx.progress));

    let color = mix(vec3(.110, .620, .455), vec3(.325, .800, .631), f32(pixel));
    return vec4(color, 1.0);
}
