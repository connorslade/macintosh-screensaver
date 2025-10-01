@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read> image: array<u32>;

struct Uniform {
    view: mat4x4f,
    image_size: vec2u,
    window_size: vec2u,

    color: vec3f,
    scale: f32,
    progress: f32,
    progress_angle: f32
}

@vertex
fn vert(@builtin(vertex_index) index: u32) -> VertexOutput {
    return VertexOutput(ctx.view * vec4(QUAD_POS[index], 0.0, 1.0), QUAD_UV[index]);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(ctx.color, 1.0) * evaluate(in.uv);
}

fn evaluate(uv: vec2f) -> f32 {
    let pos = uv * vec2f(ctx.image_size) - vec2(0.5);

    let rounded = round(pos);
    let dist = chebyshev_distance(pos - rounded);

    let progress = progress(uv);
    let edge = dist - 0.45 * saturate(progress(uv));

    let pixel = pixel(vec2u(rounded));
    let cutoff_width = fwidth(dist);
    var value = f32(!pixel) + smoothstep(-cutoff_width, cutoff_width, edge);

    return saturate(1.0 - value);
}

fn pixel(pos: vec2u) -> bool {
    let idx = pos.y * ctx.image_size.x + pos.x;
    return (image[idx / 32] & (1u << (idx % 32))) == 0;
}

fn progress(uv: vec2f) -> f32 {
    let vec = vec2(cos(ctx.progress_angle), sin(ctx.progress_angle));
    return (uv.x * vec.x + uv.y * vec.y) * 20.0 + ctx.progress;
}

fn chebyshev_distance(vec: vec2f) -> f32 {
    return max(abs(vec.x), abs(vec.y));
}
