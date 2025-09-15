use nannou::color::{Rgb, WHITE};
use crate::position::Position;

#[derive(Debug, Copy, Clone)]
pub struct Wire {
    pub start: Position,
    pub end: Position,
    pub color: Rgb<u8>,
}

impl Default for Wire {
    fn default() -> Self {
        Wire {
            start: Position::default(),
            end: Position::default(),
            color: WHITE,
        }
    }
}

#[allow(dead_code)]
impl Wire {
    pub fn new(start: Position, end: Position) -> Wire {
        Wire { start, end, color: WHITE }
    }
    
    pub fn with_color(start: Position, end: Position, color: Rgb<u8>) -> Wire {
        Wire { start, end, color }
    }
}