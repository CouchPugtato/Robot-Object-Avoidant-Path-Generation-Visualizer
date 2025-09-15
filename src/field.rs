use nannou::color::*;

use crate::wire::{Position, Wire};


const FIELD_WIDTH: f32 = 8.23;
const FIELD_LENGTH: f32 = 16.46;

pub fn field_border() -> [Wire; 9] {

    [
        // top edge
        Wire {
            start: Position { x: 0.0, y: FIELD_WIDTH, z: 0.0 },
            end: Position { x: FIELD_LENGTH/2.0, y: FIELD_WIDTH, z: 0.0 },
            color: BLUE
        },
        Wire {
            start: Position { x: FIELD_LENGTH/2.0, y: FIELD_WIDTH, z: 0.0 },
            end: Position { x: FIELD_LENGTH, y: FIELD_WIDTH, z: 0.0 },
            color: RED
        },

        // bottom edge
        Wire {
            start: Position { x: 0.0, y: 0.0, z: 0.0 },
            end: Position { x: FIELD_LENGTH/2.0, y: 0.0, z: 0.0 },
            color: BLUE
        },
        Wire {
            start: Position { x: FIELD_LENGTH/2.0, y: 0.0, z: 0.0 },
            end: Position { x: FIELD_LENGTH, y: 0.0, z: 0.0 },
            color: RED
        },

        // left edge
        Wire {
            start: Position { x: 0.0, y: 0.0, z: 0.0 },
            end: Position { x: 0.0, y: FIELD_WIDTH, z: 0.0 },
            color: BLUE
        },

        // right edge
        Wire {
            start: Position { x: FIELD_LENGTH, y: 0.0, z: 0.0 },
            end: Position { x: FIELD_LENGTH, y: FIELD_WIDTH, z: 0.0 },
            color: RED
        },
        Wire {
            start: Position { x: FIELD_LENGTH, y: FIELD_WIDTH, z: 0.0 },
            end: Position { x: FIELD_LENGTH, y: FIELD_WIDTH, z: 0.0 },
            color: YELLOW
        },

        // middle divider
        Wire {
            start: Position { x: FIELD_LENGTH/2.0, y: 0.0, z: 0.0 },
            end: Position { x: FIELD_LENGTH/2.0, y: FIELD_WIDTH, z: 0.0 },
            color: WHITE
        },

        // zero stick
        Wire {
            start: Position { x: 0.0, y: 0.0, z: 0.0 },
            end: Position { x: 0.0, y: 0.0, z: 1.0 },
            color: WHITE
        },
    ]
}