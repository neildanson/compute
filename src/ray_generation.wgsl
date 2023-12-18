struct ScreenCoordinate {
    x: f32,
    y: f32,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}


@group(0)
@binding(0)
var<storage, read_write> screen_coordinates: array<ScreenCoordinate>; 

@group(0)
@binding(1)
var<storage, read_write> result: array<Ray>; 

//@group(0)
//@binding(2)
//var<uniform> u_entity: Color;
//


@compute
@workgroup_size(256,1,1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let ray = Ray(vec3<f32>(screen_coordinates[global_id.x].x, screen_coordinates[global_id.x].y, 0.0), vec3<f32>(0.0, 0.0, 0.0));
    result[global_id.x] = ray;//screen_coordinates[global_id.x].x * screen_coordinates[global_id.x].y;
}