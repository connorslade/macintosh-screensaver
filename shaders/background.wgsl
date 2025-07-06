@group(0) @binding(0) var<uniform> ctx: Uniform;

struct Uniform {
    start: vec3f,
    end: vec3f
}

@vertex
fn vert(@builtin(vertex_index) index: u32) -> VertexOutput {
    return VertexOutput(vec4(QUAD_POS[index], 0.0, 1.0), QUAD_UV[index]);
}

// Just a linear gradient from the start to end color over the height of the screen
@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = mix(ctx.start, ctx.end, (1.0 - in.uv.y));
    return vec4(color, 1.0);
}
