use crate::position::Position;
use crate::wire::Wire;
use nannou::prelude::*;

pub struct TargetPosition {
    pub position: Position,
    pub height: f32,
    pub color: nannou::color::Rgb<u8>,
}

pub const TARGET_INITIAL_POSITION: Position = Position {
    x: 5.0,
    y: 5.0,
    z: 0.0,
};

pub const TARGET_HEIGHT: f32 = 1.0;
pub const TARGET_COLOR_R: u8 = 255;
pub const TARGET_COLOR_G: u8 = 0;
pub const TARGET_COLOR_B: u8 = 0;
pub const TARGET_MARKER_SIZE: f32 = 0.5;

impl TargetPosition {
    pub fn new(position: Position) -> Self {
        TargetPosition {
            position,
            height: TARGET_HEIGHT,
            color: nannou::color::rgb(TARGET_COLOR_R, TARGET_COLOR_G, TARGET_COLOR_B),
        }
    }
    
    pub fn create_default() -> Self {
        Self::new(TARGET_INITIAL_POSITION)
    }
    
    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }
    
    pub fn get_position(&self) -> Position {
        self.position
    }
    
    pub fn get_wires(&self) -> Vec<Wire> {
        let half_size = TARGET_MARKER_SIZE / 2.0;
        
        let x = self.position.x;
        let y = self.position.y;
        let z = self.position.z;
        
        vec![
            Wire {
                start: Position::new(x - half_size, y - half_size, z),
                end: Position::new(x + half_size, y + half_size, z),
                color: self.color,
            },
            Wire {
                start: Position::new(x - half_size, y + half_size, z),
                end: Position::new(x + half_size, y - half_size, z),
                color: self.color,
            },
            Wire {
                start: Position::new(x, y, z),
                end: Position::new(x, y, z + self.height),
                color: self.color,
            },
        ]
    }
}