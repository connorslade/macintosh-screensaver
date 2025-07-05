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
fn vert(@builtin(vertex_index) index: u32) -> VertexOutput {
    var uvs = array(
        vec2(0.0, 0.0), vec2(1.0, 0.0),
        vec2(1.0, 1.0), vec2(0.0, 1.0)
    );
    var points = array(
        vec2(-1.0, -1.0), vec2(1.0, -1.0),
        vec2(1.0, 1.0), vec2(-1.0, 1.0)
    );

    return VertexOutput(vec4(points[index], 0.0, 1.0), uvs[index]);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = mix(ctx.start, ctx.end, (1.0 - in.uv.y));
    return vec4(color, 1.0);
}
