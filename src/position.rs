use std::default::Default;

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub const ORIGIN: Position = Position { x: 0.0, y: 0.0, z: 0.0 };

impl Default for Position {
    fn default() -> Self {
        ORIGIN
    }
}

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    pub fn move_by(&mut self, dx: f32, dy: f32, dz: f32) {
        self.x += dx;
        self.y += dy;
        self.z += dz;
    }

    pub fn scalar(&mut self, scalar: f32) -> Position {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
    
    pub fn approx_equals(&self, other: &Position) -> bool {
        const EPSILON: f32 = 0.0001;
        (self.x - other.x).abs() < EPSILON &&
        (self.y - other.y).abs() < EPSILON &&
        (self.z - other.z).abs() < EPSILON
    }

    pub fn distance_to(&self, other: &Position) -> f32 {
        ((self.x - other.x).powf(2.0) + (self.y - other.y).powf(2.0)).sqrt()
    }

    pub fn dot(&self, other: &Position) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn norm2D(&self) -> Position {
        let norm = (self.x.powf(2.0) + self.y.powf(2.0)).sqrt();
        if norm <= f32::EPSILON {
            return Self { x: 0.0, y: 0.0, z: 0.0 };
        }
        Self { x: self.x / norm, y: self.y / norm, z: 0.0 }
    }

    pub fn minus(&self, other: &Position) -> Position {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}
