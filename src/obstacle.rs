use crate::model::{Model, ModelConfig};
use crate::position::Position;
use std::f32::consts::{PI,E};

pub struct Obstacle {
    pub model: Model,
    radius: f32,
    calculation_radius: f32, // includes robot radius and buffer
    b: f32,
    robot_radius: f32, // radius of the pathing robot
    buffer_radius: f32,
}

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

const DEFAULT_BUFFER_RADIUS: f32 = 0.2; 
const DEFAULT_ROBOT_RADIUS: f32 = 0.5;
const EPS: f32 = 0.00005;
const ADJUST_RATE: f32 = 0.001;

impl Obstacle {
    
    pub fn new(model: Model) -> Obstacle {
        let radius: f32 = model.config.scale/2.0;
        let robot_radius: f32 = DEFAULT_ROBOT_RADIUS;
        let buffer_radius: f32 = DEFAULT_BUFFER_RADIUS;
        let calculation_radius: f32 = radius + robot_radius + buffer_radius;
        let b: f32 = calculation_radius * PI;
        
        Obstacle {
            model,
            radius,
            calculation_radius,
            b,
            robot_radius,
            buffer_radius,
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
        self.update_calculation_radius();
    }
    
    fn update_calculation_radius(&mut self) {
        self.calculation_radius = self.radius + self.robot_radius + self.buffer_radius;
        self.b = self.calculation_radius * PI;
    }
    
    // field functions
    
    pub fn cosine_field_function(&self, pos: Position) -> f32 {
        // 2d distance away from center
        let center: Position = self.model.config.position;
        let dist: f32 = pos.distance_to(&center);
        
        if dist > self.calculation_radius {
            return 0.0;
        }
        
        // cosine function that peaks at center and falls to zero at distance self.b
        self.b/2.0 * (PI * dist / self.b).cos() //+ self.b/2.0
    }
    
    pub fn gaussian_field_function(&self, pos: Position) -> f32 {
        let center: Position = self.model.config.position;
        let dist: f32 = pos.distance_to(&center);
        
        self.b * E.powf(-(dist/self.calculation_radius))
    }
    
    pub fn cosine_gradient_function(&self, pos: Position) -> [f32; 2] {
        let center: Position = self.model.config.position;
        let dist: f32 = pos.distance_to(&center);
       
        if dist > self.calculation_radius || dist < EPS { 
            return [0.0,0.0];
        }
        
        // calculate normalized direction vector (away from obstacle center)
        let dx: f32 = (pos.x - center.x) / dist;
        let dy: f32 = (pos.y - center.y) / dist;

        // scale by gradient magnitude (derivative of height function)
        let magnitude: f32 = PI/2.0 * (PI * dist / self.b).sin() * ADJUST_RATE;
        
        [-magnitude * dx, -magnitude * dy] // negative magnitude for gradient decent
    }
    
    pub fn gaussian_gradient_function(&self, pos: Position) -> [f32; 2] {
        let center: Position = self.model.config.position;
        let dist_x: f32 = pos.x - center.x;
        let dist_y: f32 = pos.y - center.y;
        
        [ 2.0 * PI * dist_x / self.calculation_radius * E.powf(-(dist_x/self.calculation_radius)), 
          2.0 * PI * dist_y / self.calculation_radius * E.powf(-(dist_y/self.calculation_radius)) ]
    }
}