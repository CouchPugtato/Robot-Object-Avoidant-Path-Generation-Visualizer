use crate::model::{Model, ModelConfig};
use crate::position::Position;
use std::f32::consts::{PI,E};

pub struct Obstacle {
    // Extending Model through composition
    pub model: Model,
    radius: f32,
    b: f32,
}

// Implement methods to delegate to the inner Model
impl std::ops::Deref for Obstacle {
    type Target = Model;
    
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl std::ops::DerefMut for Obstacle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.model
    }
}

const BUFFER_RADIUS: f32 = 0.1034;
const EPS: f32 = 0.00005;
const ADJUST_RATE: f32 = 0.1;

impl Obstacle {
    
    pub fn new(model: Model) -> Obstacle {
        let radius: f32 = model.config.scale/2.0;
        let b: f32 = radius * PI;
        Obstacle {
            model,
            radius,
            b,
        }
    }
    
    pub fn from_config(config: &ModelConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let model = Model::from_config(config)?;
        Ok(Self::new(model))
    }
    
    pub fn get_radius(&self) -> f32 {
        self.radius
    }
    
    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
        self.b = radius * PI;
    }
    
    // field functions
    
    pub fn cosine_field_function(&self, pos: Position) -> f32 {
        // 2d distance away from center
        let center: Position = self.model.config.position;
        let dist: f32 = ((pos.x - center.x).powf(2.0) + (pos.y - center.y).powf(2.0)).sqrt();
        
        if dist > self.radius {
            return 0.0;
        }
        
        // cosine function that peaks at center and falls to zero at distance self.b
        self.b/2.0 * (PI * dist / self.b).cos() + self.b/2.0
    }
    
    pub fn gaussian_field_function(&self, pos: Position) -> f32 {
        let center: Position = self.model.config.position;
        let dist: f32 = ((pos.x - center.x).powf(2.0) + (pos.y - center.y).powf(2.0)).sqrt();
        
        self.b * E.powf(-(dist/self.radius))
    }
    
    fn cosine_gradient_function(&self, pos: Position) -> [f32; 2] {
        let center: Position = self.model.config.position;
        let dist: f32 = ((pos.x - center.x).powf(2.0) + (pos.y - center.y).powf(2.0)).sqrt();
       
        if dist > self.radius || dist < EPS {  // Avoid division by zero
            return [0.0,0.0];
        }
        
        // calculate normalized direction vector (away from obstacle center)
        let dx: f32 = (pos.x - center.x) / dist;
        let dy: f32 = (pos.y - center.y) / dist;

        // Scale by gradient magnitude (derivative of height function)
        let magnitude: f32 = PI/2.0 * (PI * dist / self.b).sin() * ADJUST_RATE;
        
        [-magnitude * dx, -magnitude * dy] // negative magnitude for gradient decent
    }
    
    pub fn gaussian_gradient_function(&self, pos: Position) -> [f32; 2] {
        let center: Position = self.model.config.position;
        let dist_x: f32 = pos.x - center.x;
        let dist_y: f32 = pos.y - center.y;
        
        [ 2.0 * PI * dist_x / self.radius * E.powf(-(dist_x/self.radius)), 2.0 * PI * dist_y / self.radius * E.powf(-(dist_y/self.radius)) ]
    }
}