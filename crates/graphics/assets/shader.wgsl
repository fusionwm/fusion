@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var t_sampler: sampler;


struct Instance {
    @location(1) uv: vec4<f32>,
    @location(2) size: vec2<f32>,

    @location(3) model_matrix_0: vec4<f32>,
    @location(4) model_matrix_1: vec4<f32>,
    @location(5) model_matrix_2: vec4<f32>,
    @location(6) model_matrix_3: vec4<f32>,
    @location(7) color: vec4<f32>,

    @location(8) stroke_color_left: vec4<f32>,
    @location(9) stroke_color_right: vec4<f32>,
    @location(10) stroke_color_top: vec4<f32>,
    @location(11) stroke_color_bottom: vec4<f32>,

    @location(12) color_end: vec4<f32>,

    //degree: f32, 
    //use_gradient: u32,
    //support_stroke: u32,
    //stroke_width: f32,
    @location(13) misc: vec4<f32>,
};


struct VertexPayload {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,

    @location(3) stroke_color_left: vec4<f32>,
    @location(4) stroke_color_right: vec4<f32>,
    @location(5) stroke_color_top: vec4<f32>,
    @location(6) stroke_color_bottom: vec4<f32>,

    @location(7) color_end: vec4<f32>,

    //degree: f32, 
    //use_gradient: u32,
    //support_stroke: u32,
    //stroke_width: f32,
    @location(8) misc: vec4<f32>,
};

struct Vertex {
    @location(0) position: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    vertex: Vertex,
    instance: Instance
) -> VertexPayload {

    let local_uv = vec2<f32>(
        mix(instance.uv.x, instance.uv.z, vertex.position.x),
        mix(instance.uv.y, instance.uv.w, vertex.position.y)
    );

    let model = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexPayload;

    out.uv = local_uv;
    out.position = model * vec4<f32>(vertex.position, 1.0);
    out.size = instance.size;
    out.color = instance.color;

    out.stroke_color_left = instance.stroke_color_left;
    out.stroke_color_right = instance.stroke_color_right;
    out.stroke_color_top = instance.stroke_color_top;
    out.stroke_color_bottom = instance.stroke_color_bottom;
    out.color_end = instance.color_end;
    out.misc = instance.misc;

    return out;
}


@fragment
fn fs_main(in: VertexPayload) -> @location(0) vec4<f32> {
    let texColor = textureSample(texture, t_sampler, in.uv);
    var baseColor = texColor * in.color;

    let degree: f32 = in.misc.x;
    let use_gradient: u32 = u32(in.misc.y);
    let support_stroke: u32 = u32(in.misc.z);
    let stroke_width: f32 = in.misc.w;

    if use_gradient >= 1u {
        let angle = degree * 3.14159265 / 180.0;
        let dir = vec2<f32>(cos(angle), sin(angle));
        let centered_uv = in.uv - vec2<f32>(0.5);
        let max_len = 0.707;
        let raw_t = dot(centered_uv, dir);
        let t = clamp((raw_t / max_len + 1.0) * 0.5, 0.0, 1.0);
        baseColor = texColor * mix(in.color, in.color_end, t);
    }

    if stroke_width > 0.0 && support_stroke >= 1u {
        let stroke_norm = vec2(
            stroke_width / in.size.x,
            stroke_width / in.size.y
        );

        let left_factor = step(in.uv.x, stroke_norm.x);
        let right_factor = step(1.0 - in.uv.x, stroke_norm.x);
        let top_factor = step(in.uv.y, stroke_norm.y);
        let bottom_factor = step(1.0 - in.uv.y, stroke_norm.y);

        if (left_factor > 0.0) {
            return in.stroke_color_left;
        } else if (right_factor > 0.0) {
            return in.stroke_color_right;
        } else if (top_factor > 0.0) {
            return in.stroke_color_top;
        } else if (bottom_factor > 0.0) {
            return in.stroke_color_bottom;
        }
    }

    return baseColor;
}

