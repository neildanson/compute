struct Pair {
    a: u32,
    b: u32,
}

struct Color {
    color: vec4<f32>,
};

@group(0)
@binding(0)
var<storage, read_write> result: array<u32>; 

@group(0)
@binding(1)
var<storage, read_write> input: array<Pair>; 


@group(0)
@binding(2)
var<uniform> u_entity: Color;



@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    result[global_id.x] = input[global_id.x].a * input[global_id.x].b * u32(u_entity.color.x);
}