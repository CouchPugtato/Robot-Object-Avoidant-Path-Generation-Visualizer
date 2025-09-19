use crate::model::{Model, ModelConfig};
use crate::position::Position;
use crate::wire::Wire;
use crate::obstacle::Obstacle;
use nannou::color::rgb;

pub struct PathPoint {
    pub position: Position,
    pub height: f32,
}

impl PathPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Position::new(x, y, 0.0),
            height: 0.0,
        }
    }
    
    pub fn from_position(position: Position) -> Self {
        Self {
            position: Position::new(position.x, position.y, 0.0),
            height: position.z,
        }
    }
    
    pub fn get_height(&self) -> f32 {
        self.height
    }
    
    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }
}

pub struct Robot {
    pub model: Model,
    pub path_points: Vec<PathPoint>,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub current_path_index: usize,
    pub target_speed: f32,
}

impl std::ops::Deref for Robot {
    type Target = Model;
    
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl std::ops::DerefMut for Robot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.model
    }
}

pub const ROBOT_INITIAL_POSITION: Position = Position {
    x: 2.0,
    y: 2.0,
    z: 0.0,
};

pub const ROBOT_MODEL_NAME: &str = "robot_base";
pub const ROBOT_SCALE: f32 = 1.0;

pub const MIN_ADJUST_RATE: f32 = 0.0001;
pub const MAX_ITERATIONS: usize = 1000;
pub const PATH_OPTIMIZATION_THRESHOLD: f32 = 0.01;

impl Robot {
    pub fn new(model: Model) -> Robot {
        Robot {
            model,
            path_points: Vec::new(),
            velocity_x: 0.0,
            velocity_y: 0.0,
            current_path_index: 0,
            target_speed: 2.0,
        }
    }
    
    pub fn set_velocity(&mut self, x: f32, y: f32) {
        self.velocity_x = x;
        self.velocity_y = y;
    }
    
    pub fn set_target_speed(&mut self, speed: f32) {
        self.target_speed = speed;
    }
    pub fn update_position(&mut self, dt: f32) {
        let new_x = self.model.config.position.x + self.velocity_x * dt;
        let new_y = self.model.config.position.y + self.velocity_y * dt;
        
        let old_position = self.model.config.position;
        
        self.model.config.position.x = new_x;
        self.model.config.position.y = new_y;
        
        let dx = new_x - old_position.x;
        let dy = new_y - old_position.y;
        
        for wire in &mut self.model.wires {
            wire.start.x += dx;
            wire.start.y += dy;
            wire.end.x += dx;
            wire.end.y += dy;
        }
        
        println!("Robot position: ({}, {}), Velocity: ({}, {})", 
                 self.model.config.position.x, 
                 self.model.config.position.y,
                 self.velocity_x,
                 self.velocity_y);
    }
    
    pub fn create_default() -> Result<Self, Box<dyn std::error::Error>> {
        let config = ModelConfig {
            name: ROBOT_MODEL_NAME.to_string(),
            position: ROBOT_INITIAL_POSITION,
            scale: ROBOT_SCALE,
        };
        
        let model = Model::from_config(&config)?;
        Ok(Self::new(model))
    }
    
    pub fn generate_path(&mut self, target_position: &Position, segments_count: usize, obstacles: &[Obstacle]) {
        let start = self.model.config.position;
        let end = *target_position;
        
        self.path_points = Vec::with_capacity(segments_count + 2);
        
        self.current_path_index = 0;
        
        self.path_points.push(PathPoint::from_position(start));
        
        let dx = (end.x - start.x) / segments_count as f32;
        let dy = (end.y - start.y) / segments_count as f32;
        
        for i in 1..=segments_count {
            let prev_point = &self.path_points[i-1];
            let mut p = PathPoint::new(
                prev_point.position.x + dx,
                prev_point.position.y + dy
            );
            
            p.height = 0.0;
            for obstacle in obstacles {
                p.height += obstacle.cosine_field_function(p.position);
            }
            
            self.path_points.push(p);
        }
        
        self.path_points.push(PathPoint::from_position(end));

        let step_distance = (dx*dx + dy*dy).sqrt();
        let _ = self.clean_path(step_distance);
    }
    
    pub fn get_path_wires(&self) -> Vec<Wire> {
        let mut wires = Vec::with_capacity(self.path_points.len() - 1);
        
        for i in 0..self.path_points.len() - 1 {
            let start = self.path_points[i].position;
            let end = self.path_points[i + 1].position;
            
            wires.push(Wire {
                start,
                end,
                color: rgb(0, 255, 0),
            });
        }
        
        wires
    }
    
    fn is_path_optimized(&self) -> bool {
        for i in 1..self.path_points.len() - 1 {
            if self.path_points[i].get_height() > PATH_OPTIMIZATION_THRESHOLD {
                return false;
            }
        }
        true
    }
    
    pub fn optimize_path(&mut self, obstacles: &[Obstacle]) {
        if self.path_points.len() <= 2 {
            return;
        }
        
        let mut iterations = 0;
        
        let first_point = &self.path_points[0].position;
        let second_point = &self.path_points[1].position;
        let original_step_distance = first_point.distance_to(second_point);
        
        while !self.is_path_optimized() && iterations < MAX_ITERATIONS {
            iterations += 1;
            self.optimize_path_single_iteration(obstacles);
            self.clean_path(original_step_distance);
        }

        println!("Path optimized in {} iterations", iterations);
    }
    
    pub fn optimize_path_single_iteration(&mut self, obstacles: &[Obstacle]) -> bool {
        if self.path_points.len() <= 2 {
            return true;
        }
        
        let mut all_points_optimized = true;
        
        for i in 1..self.path_points.len() - 1 {
            let point = &mut self.path_points[i];
            
            if point.get_height() > PATH_OPTIMIZATION_THRESHOLD {
                all_points_optimized = false;
                let mut total_dx = 0.0;
                let mut total_dy = 0.0;
                
                for obstacle in obstacles {
                    let gradient = obstacle.cosine_gradient_function(point.position);
                    total_dx -= gradient[0];
                    total_dy -= gradient[1];
                }
                
                if total_dx.abs() < MIN_ADJUST_RATE && total_dy.abs() < MIN_ADJUST_RATE {
                    if total_dx != 0.0 {
                        total_dx = total_dx.signum() * MIN_ADJUST_RATE;
                    }
                    if total_dy != 0.0 {
                        total_dy = total_dy.signum() * MIN_ADJUST_RATE;
                    }
                }
                
                point.position.x += total_dx;
                point.position.y += total_dy;
                
                let mut height = 0.0;
                for obstacle in obstacles {
                    height += obstacle.cosine_field_function(point.position);
                }
                point.set_height(height);
            }
        }
        
        return all_points_optimized;
    }

    pub fn clean_path(&mut self, original_step_distance: f32) -> bool {
        // remove points closer together than half original step distance, avoid clumping
        let threshold = original_step_distance*2.0;
        let mut i = 1;
        let mut removed_any = false;
        
        while i < self.path_points.len() - 1 {
            if self.path_points[i].position.distance_to(&self.path_points[i-1].position) < threshold {
                self.path_points.remove(i);
                removed_any = true;
            } else {
                i += 1;
            }
        }
        
        removed_any
    }
}

