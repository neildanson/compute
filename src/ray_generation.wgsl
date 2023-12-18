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
var<storage, read_write> result: array<Ray>; 

@compute
@workgroup_size(8,8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let WIDTH = 256;
    let HEIGHT = 256;
    let grid_x = i32(global_id.x); 
    let grid_y = i32(global_id.y); 
    let mx = WIDTH / 8; 
    let my = HEIGHT / 8;

    for (var x : i32 = 0; x < mx; x = x + 1) {
        for (var y : i32 = 0; y < my; y = y + 1) {
            let x_pos = grid_x * mx + x;
            let y_pos = grid_y * my + y;
            let array_pos = x_pos * WIDTH + y_pos;

            let r = f32(x_pos) / f32(WIDTH);
            let g = f32(y_pos) / f32(HEIGHT);
            let b = 0.0;

            let ray = Ray(vec4<f32>(r, g, b, 1.0), 
                          vec4<f32>(screen_coordinates[array_pos].x, screen_coordinates[array_pos].y, 0.0, 1.0));
            result[array_pos] = ray;
        }
    }
}