struct ScreenCoordinate {
    x: f32,
    y: f32,
}

struct Ray {
    origin: vec4<f32>,
    direction: vec4<f32>,
}


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

@compute
@workgroup_size(16,16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    //These are effectively constants
    let grid_size_x = 16;
    let grid_size_y = 16;
    let grid_cell_size_x = width / grid_size_x; 
    let grid_cell_size_y = height / grid_size_y;

    let grid_x = i32(global_id.x); 
    let grid_y = i32(global_id.y); 

    let start_pos = (grid_y * width * grid_size_y) + (grid_x * grid_cell_size_x);
    for (var x : i32 = 0; x < grid_cell_size_x; x = x + 1) {
        for (var y : i32 = 0; y < grid_cell_size_y; y = y + 1) {
            let array_pos = start_pos + x + (y * width);

            let r = screen_coordinates[array_pos].x / f32(width);
            let g = screen_coordinates[array_pos].y / f32(height);
            let b = 0.0;

            let ray = Ray(vec4<f32>(r, g, b, 1.0), 
                          vec4<f32>(screen_coordinates[array_pos].x, screen_coordinates[array_pos].y, 0.0, 1.0));
            result[array_pos] = ray;
        }
    }
}