struct Ray {
    origin: vec4<f32>,
    direction: vec4<f32>,
}

struct Sphere {
    position : vec3<f32>,
    radius : f32,
}

struct Intersection { 
    ray : Ray, //8
    //distance : f32,
    //sphere : Sphere,
    //is_hit : i32, //9
    padding : vec4<i32>, //12
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
    if (v < 0.0) {
        return Intersection(ray, vec4<i32>(0,0,0, 0));
    } else {
        let distance_squared = pow(sphere.radius, 2.0) - (dot(diff, diff) - pow(v,2.0));
        if (distance_squared < 0.0) {
            return Intersection (ray, vec4<i32>(0,0,0, 0));
        } else {
            let distance = v - sqrt(distance_squared);
            //hit
            return Intersection (ray, vec4<i32>(0,0,0,1));
        }
    }
}


//fn nearest_intersection(Ray ray, out Intersection nearest_intersection, out Sphere s) {
//    bool hit = false;
//    float nearest_dist = 1000000.0;
//    for (int i = 0; i < NUM_SPHERE; i++) {
//        Intersection intersection;
//        if (intersects(sphere[i], ray, intersection)) {
//            if (intersection.distance < nearest_dist) {
//                hit = true;
//                nearest_dist = intersection.distance;
//                nearest_intersection = intersection;
//                s = sphere[i];
//            }
//        }
//    }
//    return hit;
//}

@compute
@workgroup_size(256, 1)
fn main(@builtin(global_invocation_id) global_invocation_id : vec3<u32>, ) {
    let num_spheres = 1;
    let width = 640;
    let array_pos = i32(global_invocation_id.x);

    for (var j = 0; j < num_spheres; j++) {
        let sphere = spheres[j];
        result[array_pos] = intersects(sphere, input[array_pos]);
    } 
}