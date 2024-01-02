struct Ray {
    origin: vec4<f32>,
    direction: vec4<f32>,
}

struct Camera {
    position : vec3<f32>,
    forward : vec3<f32>,
    up : vec3<f32>,
    right : vec3<f32>,
}

@group(0)
@binding(1)
var<uniform> width: i32;

@group(0)
@binding(2)
var<uniform> height: i32;

@group(0)
@binding(3)
var<storage, read_write> result: array<Ray>; 

fn create_camera(position : vec3<f32>, look_at : vec3<f32>, height : f32) -> Camera {    
    let forward = normalize(look_at - position);
    let down = vec3(0.0,-1.0,0.0);
    let right = normalize(cross(forward,down)) * 1.5 / height;
    let up =  normalize(cross(forward,right)) * 1.5 / height;
    let camera = Camera(position, forward, up, right);
    return camera;
}

fn recenter_x(x : f32, half_width : f32) -> f32 {
    return x - half_width;
}

fn recenter_y(y : f32, half_height : f32) -> f32 {
    return (y - half_height);
}

fn get_ray(x : f32, y : f32, half_width : f32, half_height :f32, camera : Camera) -> Ray {
    let right = camera.right * (recenter_x(x, half_width));
    let up = camera.up * (recenter_y(y, half_height));
    let ray = Ray(vec4(camera.position, 1.0), vec4(normalize(right + up + camera.forward), 1.0));
    return ray;
}

@compute
@workgroup_size(256,1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let half_width = f32(width) / 2.0;
    let half_height = f32(height) / 2.0;
    let camera = create_camera(vec3<f32>(0.0,0.0,0.0), vec3<f32>(0.0,0.0,1.0), f32(height));

    let array_pos =  i32(global_id.x);
    let x = array_pos % width;
    let y = array_pos / width;
    let ray = 
        get_ray(
            f32(x), f32(y), 
            half_width, half_height, 
            camera);

    result[array_pos] = ray;    
}