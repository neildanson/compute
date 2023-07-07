struct Pair {
    a: u32,
    b: u32,
}


@group(0)
@binding(0)
var<storage, read_write> result: array<u32>; 


@group(0)
@binding(1)
var<storage, read_write> v_indices1: array<Pair>; 



@group(0)
@binding(2)
var<storage, read_write> v_indices2: array<u32>;// this is used as both input and output for convenience

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    result[global_id.x] = v_indices1[global_id.x].a + v_indices1[global_id.x].b;
}