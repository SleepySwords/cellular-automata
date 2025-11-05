struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec3<f32>,
    @location(1) colour: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VertexInput {
    @location(0) vert_pos: vec3<f32>,
    @location(1) colour: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct Camera {
    scale: f32,
    x: f32,
    y: f32
}

@group(1) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = /* camera.view_proj * */ vec4((model.vert_pos.xy + vec2(camera.x, camera.y)) * camera.scale, model.vert_pos.z, 1.0);
    out.tex_coords = model.tex_coords;
    out.vert_pos = model.vert_pos.xyz;
    out.colour = model.colour;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

@group(0) @binding(2)
var t_output: texture_storage_2d<rgba8unorm, write>;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}

@compute
@workgroup_size(1, 1, 1)
fn cm_main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>
) {
    let x = global_invocation_id.x;
    let y = global_invocation_id.y;

    var count = 0;

    for (var i: i32 = -1; i <= 1; i++) {
        for (var j: i32 = -1; j <= 1; j++) {
            if i == 0 && j == 0 {
                continue;
            }
            var color = textureLoad(t_diffuse, vec2<i32>(i32(x) + i, i32(y) + j), 0);
            if color.x > 0.5f {
                count += 1;
            }
        }
    }


    var color = textureLoad(t_diffuse, vec2<i32>(i32(x), i32(y)), 0);

    if count == 3 {
        color.x = 1.0f;
        color.y = 1.0f;
        color.z = 1.0f;
    } else if count < 2 || count > 3 {
        color.x = 0.0f;
        color.y = 0.0f;
        color.z = 0.0f;
    }

    textureStore(t_output, vec2<i32>(i32(x), i32(y)), color);
}
