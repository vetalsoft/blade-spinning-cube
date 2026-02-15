struct Globals {
    mvp_matrix: mat4x4<f32>,
    model_matrix: mat4x4<f32>,
    light_pos: vec4<f32>,
    light_color: vec4<f32>,
    ambient: vec4<f32>,
    specular_power: f32,
    specular_intensity: f32,
};

var<uniform> globals: Globals;

struct VertexInput {
    pos: vec3<f32>,
    normal: vec3<f32>,
    color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) world_normal: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    let world_pos = (globals.model_matrix * vec4<f32>(input.pos, 1.0)).xyz;
    let world_normal = (globals.model_matrix * vec4<f32>(input.normal, 0.0)).xyz;
    
    output.position = globals.mvp_matrix * vec4<f32>(input.pos, 1.0);
    output.world_pos = world_pos;
    output.world_normal = normalize(world_normal);
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(globals.light_pos.xyz - input.world_pos);
    let view_dir = normalize(-input.world_pos);
    let half_dir = normalize(light_dir + view_dir);
    
    let diffuse = max(dot(input.world_normal, light_dir), 0.0);
    let specular = pow(max(dot(input.world_normal, half_dir), 0.0), globals.specular_power);
    
    let ambient_term = globals.ambient.xyz;
    let diffuse_term = diffuse * globals.light_color.xyz;
    let specular_term = specular * globals.specular_intensity * globals.light_color.xyz;

    let lit_color = input.color * (ambient_term + diffuse_term) + specular_term;

    return vec4<f32>(lit_color, 1.0);
}