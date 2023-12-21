struct ScreenCoordinate {
    x: f32,
    y: f32,
}

struct Ray {
    origin: vec4<f32>,
    direction: vec4<f32>,
    screen_coordinates: ScreenCoordinate,
}

struct Sphere {
    position : vec3<f32>,
    radius : f32,
    
}

struct Intersection { 
    ray : Ray,
    //distance : f32,
    //sphere : Sphere,
    is_hit : i32,
    _padding : vec3<i32>,
}





@group(0)
@binding(0)
var<storage, read> spheres: array<Sphere>; 

@group(0)
@binding(1)
var<storage, read_write> result: array<Intersection>; 

@group(0)
@binding(3)
var<storage, read> input: array<Ray>; 

fn intersects(sphere : Sphere, ray : Ray) -> Intersection {
    let diff = sphere.position.xyz - ray.origin.xyz;
    let v = dot(diff, ray.direction.xyz);
    if (v < 0.0) {
        return Intersection(ray, 0, vec3<i32>(0,0,0));
    } else {
        let distance_squared = pow(sphere.radius, 2.0) - (dot(diff, diff) - pow(v,2.0));
        if (distance_squared < 0.0) {
            return Intersection (ray, 0, vec3<i32>(0,0,0));
        } else {
            let distance = v - sqrt(distance_squared);

            return Intersection (ray, 1, vec3<i32>(0,0,0));
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
@workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let num_spheres = 1;
    let grid_size_x = 16;
    let grid_size_y = 16;
    let width = 1920;
    let height = 1080;
    let grid_cell_size_x = width / grid_size_x; 
    let grid_cell_size_y = height / grid_size_y;

    let grid_x = i32(global_id.x); 
    let grid_y = i32(global_id.y); 

    let start_pos = (grid_y * width * grid_size_y) + (grid_x * grid_cell_size_x);

    for (var x : i32 = 0; x < grid_cell_size_x; x = x + 1) {
        for (var y : i32 = 0; y < grid_cell_size_y; y = y + 1) {
            let array_pos = start_pos + x + (y * width);

            for (var i = 0; i < num_spheres; i++) {
                let sphere = spheres[i];
                let intersection = intersects(sphere, input[array_pos]);
                if (intersection.is_hit == 1) {
                    result[array_pos] = intersection;
                }
            }
        }
    }

    
}