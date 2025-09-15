#[derive(Default, Debug, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub const ORIGIN: Position = Position {x: 0.0, y: 0.0, z: 0.0};

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Position {
        Position { x, y, z }
    }

    pub fn move_by(&mut self, dx: f32, dy: f32, dz: f32) {
        self.x += dx;
        self.y += dy;
        self.z += dz;
    }
    
    pub fn approx_equals(&self, other: &Position) -> bool {
        const EPS: f32 = 0.0001;
        (self.x - other.x).abs() < EPS &&
        (self.y - other.y).abs() < EPS &&
        (self.z - other.z).abs() < EPS
    }
}


#[derive(Debug, Copy, Clone)]
pub struct Wire {
    pub start: Position,
    pub end: Position,
    pub color: nannou::color::Rgb<u8>,
}