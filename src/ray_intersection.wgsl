struct Ray {
    origin: vec4<f32>,
    direction: vec4<f32>,
}

struct Sphere {
    position : vec3<f32>,
    radius : f32,
}

//should contain ray, sphere, distance, is_hit
struct Intersection { 
    ray : Ray, //32
    sphere : Sphere, //64
    hit_data : vec4<f32>, //80
}

@group(0)
@binding(0)
var<storage, read> input: array<Ray>; 

@group(0)
@binding(1)
var<storage, read> spheres: array<Sphere>; 

@group(0)
@binding(2)
var<storage, read_write> result: array<Intersection>; 

fn intersects(sphere : Sphere, ray : Ray) -> Intersection {
    let diff = sphere.position.xyz - ray.origin.xyz;
    let v = dot(diff, ray.direction.xyz);
    if v < 0.0 {
        return Intersection(ray, sphere, vec4<f32>(0.0));
    } else {
        let distance_squared = pow(sphere.radius, 2.0) - (dot(diff, diff) - pow(v,2.0));
        if distance_squared < 0.0 {
            return Intersection (ray, sphere, vec4<f32>(0.0));
        } else {
            let distance = v - sqrt(distance_squared);
            return Intersection (ray,sphere, vec4<f32>(1.0, distance, 0.0, 0.0));
        }
    }
}

@compute
@workgroup_size(256, 1)
fn main(@builtin(global_invocation_id) global_invocation_id : vec3<u32>, ) {
    let num_spheres = i32(arrayLength(&spheres));
    let width = 640;
    let array_pos = i32(global_invocation_id.x);

    var nearest_distance = 1000000.0;
    for (var j = 0; j < num_spheres; j++) {
        let sphere = spheres[j];
        let new_intersection = intersects(sphere, input[array_pos]);
        if new_intersection.hit_data.x > 0.0 
            && new_intersection.hit_data.y < nearest_distance 
            {
            nearest_distance = new_intersection.hit_data.y; 
            result[array_pos] = new_intersection;
        }
    } 
}