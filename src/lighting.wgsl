struct Ray {
    origin: vec4<f32>,
    direction: vec4<f32>,
}

struct Sphere {
    position : vec3<f32>,
    radius : f32,
}

struct Intersection { 
    ray : Ray, //32
    sphere : Sphere, //64
    hit_data : vec4<f32>, //80
}

@group(0)
@binding(0)
var<storage, read> intersections: array<Intersection>; 

@group(0)
@binding(1)
var<storage, read_write> result: array<i32>; 

@compute
@workgroup_size(256, 1)
fn main(@builtin(global_invocation_id) global_invocation_id : vec3<u32>, ) {
    let intersection = intersections[global_invocation_id.x];
    if (intersection.hit_data.x > 0.0) {
        result[global_invocation_id.x] = 1;
    }
    else {
        result[global_invocation_id.x] = 0;
    }
}