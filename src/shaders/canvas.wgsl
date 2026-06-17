struct Viewport {
    size: vec2<f32>,
};
@group(0) @binding(0) var<uniform> viewport: Viewport;

@group(1) @binding(0) var canvas_tex: texture_2d<f32>;
@group(1) @binding(1) var canvas_sampler: sampler;

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_pos = vec4<f32>(
        (input.pos.x / viewport.size.x) * 2.0 - 1.0,
        -((input.pos.y / viewport.size.y) * 2.0 - 1.0),
        0.0,
        1.0,
    );
    out.uv = input.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(canvas_tex, canvas_sampler, input.uv);
}
