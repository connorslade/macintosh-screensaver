@group(0) @binding(0) var<uniform> ctx: Uniform;

struct Uniform {
    start: vec3f,
    end: vec3f
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
    return VertexOutput(pos, uv);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = mix(ctx.start, ctx.end, (1.0 - in.uv.y));
    return vec4(color, 1.0);
}
