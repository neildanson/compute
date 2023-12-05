struct Pair {
    a: u32,
    b: u32,
}

struct Entity {
    world: mat4x4<f32>,
    color: vec4<f32>,
};

@group(1)
@binding(0)
var<uniform> u_entity: Entity;

@group(0)
@binding(0)
var<storage, read_write> result: array<u32>; 


@group(0)
@binding(1)
var<storage, read_write> v_indices1: array<Pair>; 




@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    result[global_id.x] = v_indices1[global_id.x].a * v_indices1[global_id.x].b;
}