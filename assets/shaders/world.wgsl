struct CameraUniform {
    projection_matrix: mat4x4<f32>,
    transformation_matrix: mat4x4<f32>
}

struct AtlasUniform {
    rows: u32,
    columns: u32
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var texture_atlas: texture_2d<f32>;

@group(1) @binding(1)
var atlas_sampler: sampler;

@group(1) @binding(2)
var<uniform> atlas: AtlasUniform;

@group(2) @binding(0)
var<uniform> transformation: vec3<i32>;

struct VertexInput {
    @location(0) packed: u32,
    @builtin(vertex_index) vertex_index: u32
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,

    @location(0) uv: vec2<f32>,
    @location(1) ao: f32,
    @location(2) debug: vec3<f32>
}

fn calculate_uv(
    texture_id: u32,
    vertex_index: u32
) -> vec2<f32> {
    let texture_width = 1.0 / f32(atlas.columns);
    let texture_height = 1.0 / f32(atlas.rows);

    let column = f32(texture_id % atlas.columns);
    let row = f32(texture_id / atlas.columns); 

    switch (vertex_index % 4u) {
        case 0u: {
            return vec2<f32>(column * texture_width, row * texture_height);
        }
        case 1u: {
            return vec2<f32>((column + 1.0) * texture_width, row * texture_height);
        }
        case 2u: {
            return vec2<f32>((column + 1.0) * texture_width, (row + 1.0) * texture_height);
        }
        case 3u, default: {
            return vec2<f32>(column * texture_width, (row + 1.0) * texture_height);
        }
    }
}

var<private> ao_lerps: vec4<f32> = vec4<f32>(0.1, 0.25, 0.5, 1.0);

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let transformation = vec3<f32>(transformation * 16);

    let x = f32((in.packed >> 27) & 0x1f);
    let y = f32((in.packed >> 22) & 0x1f);
    let z = f32((in.packed >> 17) & 0x1f);

    let ao_value = (in.packed >> 15) & 0x3;
    let texture_id = (in.packed >> 9) & 0x3f;
    let direction = (in.packed >> 6) & 0x7;

    out.uv = calculate_uv(texture_id, in.vertex_index);
    out.clip_position = camera.projection_matrix * camera.transformation_matrix * vec4<f32>(transformation + vec3<f32>(x, y, z), 1.0);
    out.ao = ao_lerps[ao_value];

    switch direction {
        case 0u: {
            out.debug = vec3<f32>(0.25);
        }
        case 1u: {
            out.debug = vec3<f32>(0.25);
        }
        case 2u: {
            out.debug = vec3<f32>(0.25);
        }
        case 3u: {
            out.debug = vec3<f32>(1.0, 0.0, 0.0);
        }
        case 4u: {
            out.debug = vec3<f32>(0.25);
        }
        case 5u, default: {
            out.debug = vec3<f32>(0.25);
        }         

    }

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(texture_atlas, atlas_sampler, in.uv);

    return vec4<f32>(color.rgb * in.ao, color.a);
}