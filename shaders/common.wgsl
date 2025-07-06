struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

const QUAD_UV: array<vec2f, 4> = array(
    vec2(0.0, 0.0), vec2(1.0, 0.0),
    vec2(1.0, 1.0), vec2(0.0, 1.0)
);
const QUAD_POS: array<vec2f, 4> = array(
    vec2(-1.0, -1.0), vec2(1.0, -1.0),
    vec2(1.0, 1.0), vec2(-1.0, 1.0)
);

fn invMix(a: f32, b: f32, value: f32) -> f32 {
    return (value - a) / (b - a);
}

fn remap(a: f32, b: f32, x: f32, y: f32, value: f32) -> f32 {
    let t = invMix(a, b, value);
    return mix(x, y, t);
}
