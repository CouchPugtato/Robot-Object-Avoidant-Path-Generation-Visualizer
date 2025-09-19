use crate::position::Position;
use crate::wire::Wire;
use crate::field::{FIELD_LENGTH,FIELD_WIDTH};

pub static mut OBSTACLES_REF: Option<*const Vec<crate::obstacle::Obstacle>> = None;

pub fn set_obstacles_ref(obstacles: &Vec<crate::obstacle::Obstacle>) {
    unsafe {
        OBSTACLES_REF = Some(obstacles as *const Vec<crate::obstacle::Obstacle>);
    }
}

pub fn obstacle_sum(x: f32, y: f32) -> f32 {
    unsafe {
        if let Some(obstacles_ptr) = OBSTACLES_REF {
            let obstacles = &*obstacles_ptr;
            let pos = crate::position::Position::new(x, y, 0.0);
            let mut sum = 0.0;
            
            for obstacle in obstacles {
                sum += obstacle.cosine_field_function(pos);
            }
            
            return sum;
        }
        0.0
    }
}

pub struct GradientWire {
    x_wires: Vec<Vec<Wire>>,
    y_wires: Vec<Vec<Wire>>,
    pub gradient_function: fn(f32, f32) -> f32,
    pub x_resolution: f32,
    pub y_resolution: f32,
    pub line_resolution: f32,
    color: Option<nannou::color::Rgb<u8>>,
}

impl GradientWire {
    pub fn new(gradient_func: fn(f32,f32) -> f32, x_resolution: f32, y_resolution: f32, line_resolution: f32) -> GradientWire {
        let gradient_field: [Vec<Vec<Wire>>; 2] = generate_gradient_wires(gradient_func, x_resolution, y_resolution, line_resolution);
        GradientWire { 
            x_wires: gradient_field[0].clone(),
            y_wires: gradient_field[1].clone(),
            gradient_function: gradient_func,
            x_resolution: x_resolution,
            y_resolution: y_resolution,
            line_resolution: line_resolution,
            color: None,
        }
    }
    
    pub fn with_color(mut self, color: nannou::color::Rgb<u8>) -> Self {
        self.color = Some(color);
        self.apply_color();
        self
    }
    
    pub fn set_color(&mut self, color: nannou::color::Rgb<u8>) {
        self.color = Some(color);
        self.apply_color();
    }
    
    fn apply_color(&mut self) {
        if let Some(color) = self.color {
            for wire_set in &mut self.x_wires {
                for wire in wire_set {
                    wire.color = color;
                }
            }
            
            for wire_set in &mut self.y_wires {
                for wire in wire_set {
                    wire.color = color;
                }
            }
        }
    }
    
    pub fn get_all_wires(&self) -> Vec<Wire> {
        let mut all_wires = Vec::new();

        for wire_set in &self.x_wires {
            for wire in wire_set {
                all_wires.push(*wire);
            }
        }
        
        for wire_set in &self.y_wires {
            for wire in wire_set {
                all_wires.push(*wire);
            }
        }
        
        all_wires
    }
    
    pub fn update(&mut self) {
        let gradient_field = generate_gradient_wires(
            self.gradient_function, 
            self.x_resolution, 
            self.y_resolution, 
            self.line_resolution
        );
        
        self.x_wires = gradient_field[0].clone();
        self.y_wires = gradient_field[1].clone();
        
        if self.color.is_some() {
            self.apply_color();
        }
    }
}


fn generate_gradient_wires(gradient_func: fn(f32,f32) -> f32, x_resolution: f32, y_resolution: f32, line_resolution: f32) -> [Vec<Vec<Wire>>; 2] {
    let mut x_wires: Vec<Vec<Wire>> = Vec::new();
    let mut y_wires: Vec<Vec<Wire>> = Vec::new();
    
    let safe_x_resolution = x_resolution.max(0.01);
    let safe_y_resolution = y_resolution.max(0.01);
    let safe_line_resolution = line_resolution.max(0.01);
    
    let x_line_count = (FIELD_LENGTH * safe_x_resolution).max(1.0) as usize;
    let y_line_count = (FIELD_WIDTH * safe_y_resolution).max(1.0) as usize;
    
    let x_spacing = FIELD_LENGTH / x_line_count as f32;
    let y_spacing = FIELD_WIDTH / y_line_count as f32;
    
    let max_segment_length = safe_line_resolution * 5.0;
    
    for i in 0..x_line_count {
        let x = i as f32 * x_spacing;
        let mut x_wire_set: Vec<Wire> = Vec::new();
        
        let y_segments = (FIELD_WIDTH / max_segment_length).ceil() as usize;
        let y_segment_size = FIELD_WIDTH / y_segments as f32;
        
        for j in 0..y_segments {
            let y_start = j as f32 * y_segment_size;
            let y_end = if j == y_segments - 1 { FIELD_WIDTH } else { (j + 1) as f32 * y_segment_size };
            
            let z_start = gradient_func(x, y_start);
            let z_end = gradient_func(x, y_end);
            
            x_wire_set.push(Wire::new(
                Position::new(x, y_start, z_start),
                Position::new(x, y_end, z_end)
            ));
        }
        
        if !x_wire_set.is_empty() {
            x_wires.push(x_wire_set);
        }
    }
    
    for i in 0..y_line_count {
        let y = i as f32 * y_spacing;
        let mut y_wire_set: Vec<Wire> = Vec::new();
        
        let x_segments = (FIELD_LENGTH / max_segment_length).ceil() as usize;
        let x_segment_size = FIELD_LENGTH / x_segments as f32;
        
        for j in 0..x_segments {
            let x_start = j as f32 * x_segment_size;
            let x_end = if j == x_segments - 1 { FIELD_LENGTH } else { (j + 1) as f32 * x_segment_size };
            
            let z_start = gradient_func(x_start, y);
            let z_end = gradient_func(x_end, y);
            
            y_wire_set.push(Wire::new(
                Position::new(x_start, y, z_start),
                Position::new(x_end, y, z_end)
            ));
        }
        
        if !y_wire_set.is_empty() {
            y_wires.push(y_wire_set);
        }
    }
    
    let mut final_x_wire_set: Vec<Wire> = Vec::new();
    let x = FIELD_LENGTH;
    
    let y_segments = (FIELD_WIDTH / max_segment_length).ceil() as usize;
    let y_segment_size = FIELD_WIDTH / y_segments as f32;
    
    for j in 0..y_segments {
        let y_start = j as f32 * y_segment_size;
        let y_end = if j == y_segments - 1 { FIELD_WIDTH } else { (j + 1) as f32 * y_segment_size };
        
        let z_start = gradient_func(x, y_start);
        let z_end = gradient_func(x, y_end);
        
        final_x_wire_set.push(Wire::new(
            Position::new(x, y_start, z_start),
            Position::new(x, y_end, z_end)
        ));
    }
    
    if !final_x_wire_set.is_empty() {
        x_wires.push(final_x_wire_set);
    }
    
    let mut final_y_wire_set: Vec<Wire> = Vec::new();
    let y = FIELD_WIDTH;
    
    let x_segments = (FIELD_LENGTH / max_segment_length).ceil() as usize;
    let x_segment_size = FIELD_LENGTH / x_segments as f32;
    
    for j in 0..x_segments {
        let x_start = j as f32 * x_segment_size;
        let x_end = if j == x_segments - 1 { FIELD_LENGTH } else { (j + 1) as f32 * x_segment_size };
        
        let z_start = gradient_func(x_start, y);
        let z_end = gradient_func(x_end, y);
        
        final_y_wire_set.push(Wire::new(
            Position::new(x_start, y, z_start),
            Position::new(x_end, y, z_end)
        ));
    }
    
    if !final_y_wire_set.is_empty() {
        y_wires.push(final_y_wire_set);
    }
    
    [x_wires, y_wires]
}