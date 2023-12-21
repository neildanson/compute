struct ScreenCoordinate {
    x: f32,
    y: f32,
}

struct Ray {
    origin: vec4<f32>,
    direction: vec4<f32>,
    screen_coordinates: ScreenCoordinate,
}

struct Camera {
    position : vec3<f32>,
    forward : vec3<f32>,
    up : vec3<f32>,
    right : vec3<f32>,
};


@group(0)
@binding(0)
var<storage, read> screen_coordinates: array<ScreenCoordinate>; 

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

fn get_ray(screen_coordinate : ScreenCoordinate, half_width : f32, half_height :f32, camera : Camera) -> Ray {
    let right = camera.right * (recenter_x(screen_coordinate.x, half_width));
    let up = camera.up * (recenter_y(screen_coordinate.y, half_height));
    let ray = Ray(vec4(camera.position, 1.0), vec4(normalize(right + up + camera.forward), 1.0), screen_coordinate);
    return ray;
}


@compute
@workgroup_size(8,8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    //These are effectively constants
    let grid_size_x = 8;
    let grid_size_y = 8;
    let grid_cell_size_x = width / grid_size_x; 
    let grid_cell_size_y = height / grid_size_y;

    let grid_x = i32(global_id.x); 
    let grid_y = i32(global_id.y); 

    let start_pos = (grid_y * width * grid_size_y) + (grid_x * grid_cell_size_x);

    let half_width = f32(width) / 2.0;
    let half_height = f32(height) / 2.0;
    let camera = create_camera(vec3<f32>(0.0,0.0,0.0), vec3<f32>(0.0,0.0,1.0), f32(height));

    for (var x : i32 = 0; x < grid_cell_size_x; x = x + 1) {
        for (var y : i32 = 0; y < grid_cell_size_y; y = y + 1) {
            let array_pos = start_pos + x + (y * width);

            let ray = 
                get_ray(
                    screen_coordinates[array_pos], 
                    half_width, half_height, 
                    camera);

            result[array_pos] = ray;
        }
    }
}