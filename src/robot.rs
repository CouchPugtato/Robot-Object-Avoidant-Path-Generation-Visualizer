use crate::model::{Model, ModelConfig};
use crate::position::{self, Position, ORIGIN};
use crate::wire::Wire;
use crate::obstacle::Obstacle;
use clearscreen;
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
    pub current_path_progress: f32,
    pub target_speed: f32,
    pub follow_path: bool,
    pub velocity_update_timer: f32,
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
pub const MAX_ITERATIONS: usize = 2000;
pub const PATH_OPTIMIZATION_THRESHOLD: f32 = 0.001;

impl Robot {
    pub fn new(model: Model) -> Robot {
        Robot {
            model,
            path_points: Vec::new(),
            velocity_x: 0.0,
            velocity_y: 0.0,
            current_path_progress: 0.0,
            target_speed: 2.0,
            follow_path: false,
            velocity_update_timer: 0.0,
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
        self.velocity_update_timer += dt;
        
        if self.follow_path {
            self.follow_path_with_dt(dt);
        }
        
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
        
        self.current_path_progress = 0.0;
        
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
        self.optimize_path(obstacles);
        self.get_points_of_curvature(); // remove points that are too sharp of a turn
    }
    
    pub fn get_path_wires(&self) -> Vec<Wire> {
        if self.path_points.len() < 2 {
            return Vec::new();
        }
        
        let spline_points = self.generate_catmull_rom_spline();
        let mut wires = Vec::with_capacity(spline_points.len() - 1);
        
        for i in 0..spline_points.len() - 1 {
            let start = spline_points[i];
            let end = spline_points[i + 1];
            
            wires.push(Wire {
                start,
                end,
                color: rgb(0, 255, 0),
            });
        }
        
        wires
    }
    
    fn generate_catmull_rom_spline(&self) -> Vec<Position> {
        let segments = self.path_points.len();

        if segments < 2 {
            return Vec::new();
        }
        
        let mut result = Vec::new();
        let n = self.path_points.len();
        
        result.push(self.path_points[0].position);
        
        for i in 0..n-1 {
            let p0 = if i == 0 { self.path_points[0].position } else { self.path_points[i-1].position };
            let p1 = self.path_points[i].position;
            let p2 = self.path_points[i+1].position;
            let p3 = if i+2 >= n { self.path_points[n-1].position } else { self.path_points[i+2].position };
            
            for j in 1..=segments {
                let t = j as f32 / segments as f32;
                let point = self.catmull_rom_point(p0, p1, p2, p3, t);
                result.push(point);
            }
        }
        
        result
    }
    
    fn catmull_rom_point(&self, p0: Position, p1: Position, p2: Position, p3: Position, t: f32) -> Position {
        // matrix coefficients
        let t2 = t * t;
        let t3 = t2 * t;
        
        // blending functions
        let b0 = 0.5 * (-t3 + 2.0*t2 - t);
        let b1 = 0.5 * (3.0*t3 - 5.0*t2 + 2.0);
        let b2 = 0.5 * (-3.0*t3 + 4.0*t2 + t);
        let b3 = 0.5 * (t3 - t2);
        
        // calculate the interpolated point
        let x = b0 * p0.x + b1 * p1.x + b2 * p2.x + b3 * p3.x;
        let y = b0 * p0.y + b1 * p1.y + b2 * p2.y + b3 * p3.y;
        let z = b0 * p0.z + b1 * p1.z + b2 * p2.z + b3 * p3.z;
        
        Position::new(x, y, z)
    }
    
    /// returns the x and y coordinates for a point on a Catmull-Rom spline at t (0-1)
    pub fn catmull_rom_spline(&self, t: f32) -> (f32, f32) {
        if self.path_points.len() < 4 {
            if self.path_points.is_empty() {
                return (0.0, 0.0);
            } else {
                let pos = self.path_points[0].position;
                return (pos.x, pos.y);
            }
        }
        
        let num_segments = self.path_points.len() - 3;
        let segment_t = t * num_segments as f32;
        let segment_idx = segment_t.floor() as usize;
        let local_t = segment_t - segment_idx as f32;
        
        let segment_idx = segment_idx.min(num_segments - 1);
        
        let p0 = self.path_points[segment_idx].position;
        let p1 = self.path_points[segment_idx + 1].position;
        let p2 = self.path_points[segment_idx + 2].position;
        let p3 = self.path_points[segment_idx + 3].position;
        
        let point = self.catmull_rom_point(p0, p1, p2, p3, local_t);
        
        (point.x, point.y)
    }
    
    fn is_path_optimized(&self, obstacles: &[Obstacle]) -> bool {
        for i in 1..self.path_points.len() - 1 {
            if self.path_points[i].get_height() > PATH_OPTIMIZATION_THRESHOLD {
                return false;
            }
            
            for obstacle in obstacles {
                let obstacle_pos = obstacle.model.config.position;
                let min_safe_distance = obstacle.get_radius() + 0.1; // Add small buffer
                
                let dist = self.path_points[i].position.distance_to(&obstacle_pos);
                if dist < min_safe_distance {
                    return false;
                }
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
        
        while !self.is_path_optimized(obstacles) && iterations < MAX_ITERATIONS {
            iterations += 1;
            self.optimize_path_single_iteration(obstacles);
            self.clean_path(original_step_distance);
        }
        
        for point in &mut self.path_points {
            point.position.z = 0.0;
        }

        if iterations >= MAX_ITERATIONS && !self.is_path_optimized(obstacles) {
            println!("Warning: Path optimization did not converge after {} iterations", MAX_ITERATIONS);
            
            for i in 1..self.path_points.len() - 1 {
                if self.path_points[i].get_height() > PATH_OPTIMIZATION_THRESHOLD {
                    let mut nearest_obstacle_idx = 0;
                    let mut min_dist = f32::MAX;
                    
                    for (idx, obstacle) in obstacles.iter().enumerate() {
                        let dist = self.path_points[i].position.distance_to(&obstacle.model.config.position);
                        if dist < min_dist {
                            min_dist = dist;
                            nearest_obstacle_idx = idx;
                        }
                    }
                    
                    if obstacles.len() > 0 {
                        let obstacle_pos = obstacles[nearest_obstacle_idx].model.config.position;
                        let point_pos = &mut self.path_points[i].position;
                        
                        let dx = point_pos.x - obstacle_pos.x;
                        let dy = point_pos.y - obstacle_pos.y;
                        let dist = (dx*dx + dy*dy).sqrt();
                        
                        if dist > 0.001 {
                            let nx = dx / dist;
                            let ny = dy / dist;
                            
                            point_pos.x += nx * 0.5;
                            point_pos.y += ny * 0.5;
                            
                            let mut height = 0.0;
                            for obstacle in obstacles {
                                height += obstacle.cosine_field_function(*point_pos);
                            }
                            self.path_points[i].set_height(height);
                        }
                    }
                }
            }
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
            let mut too_close_to_obstacle = false;
            
            for obstacle in obstacles {
                let obstacle_pos = obstacle.model.config.position;
                let min_safe_distance = obstacle.get_radius() + 0.1;
                
                let dist = point.position.distance_to(&obstacle_pos);
                if dist < min_safe_distance {
                    too_close_to_obstacle = true;
                    break;
                }
            }
            
            if point.get_height() > PATH_OPTIMIZATION_THRESHOLD || too_close_to_obstacle {
                all_points_optimized = false;
                let mut total_delta = Position::new(0.0, 0.0, 0.0);
                
                for obstacle in obstacles {
                    let gradient = obstacle.cosine_gradient_function(point.position);
                    
                    let obstacle_pos = obstacle.model.config.position;
                    let dist = point.position.distance_to(&obstacle_pos);
                    let min_safe_distance = obstacle.get_radius() + 0.1;
                    
                    if dist < min_safe_distance {
                        let diff = point.position.minus(&obstacle_pos);
                        if diff.norm2D() > 0.001 {
                            let mut push = diff.norm2D();
                            total_delta = total_delta.minus(&push.scalar(0.5));
                        }
                    } else {
                        let grad = Position::new(gradient[0], gradient[1], 0.0);
                        total_delta = total_delta.minus(&grad);
                    }
                }
                
                if total_delta.x.abs() < MIN_ADJUST_RATE && total_delta.y.abs() < MIN_ADJUST_RATE {
                    if total_delta.x != 0.0 {
                        total_delta.x = total_delta.x.signum() * MIN_ADJUST_RATE;
                    }
                    if total_delta.y != 0.0 {
                        total_delta.y = total_delta.y.signum() * MIN_ADJUST_RATE;
                    }
                }
                
                point.position.move_by(total_delta.x, total_delta.y, 0.0);
                
                let mut height = 0.0;
                for obstacle in obstacles {
                    height += obstacle.cosine_field_function(point.position);
                }
                point.set_height(height);
            }
        }
        
        return all_points_optimized;
    }

    fn clean_path(&mut self, original_step_distance: f32) -> bool {
        // remove points closer together than 1.3x step distance, avoid clumping
        let threshold = original_step_distance * 1.3;
        let mut i = 1;
        let mut removed_any = false;
        // clearscreen::clear().expect("failedtoclear");
        
        while i < self.path_points.len() - 1 {
            if self.path_points.len() <= 12 { break; }
            if i == 0 { i = 1; continue; }
            if self.path_points[i].position.distance_to(&self.path_points[i-1].position) < threshold {
                self.path_points.remove(i);
                removed_any = true;
            } else {
                i += 1;
            }
        }
        removed_any
    }

    // returns a Vec of points that are at the beginning or end of the path curving
    fn get_points_of_curvature(&mut self) {
        let mut i = 2;
        let mut points: Vec<usize> = Vec::new();
        
        while i < self.path_points.len() - 2 { // global alignment check for rejoin pruning
            let pos_i = self.path_points[0].position;
            if i == 0 { i = 2; }
            let v1 = self.path_points[i-1].position.minus(&pos_i);
            let v2 = self.path_points[i].position.minus(&pos_i); // central point
            let v3 = self.path_points[i+1].position.minus(&pos_i);
            let v_path = self.path_points[self.path_points.len()-1].position.minus(&pos_i);

            let is_colinear_to_path = |v: &Position| {
                if v.approx_equals(&ORIGIN) { return false; } // check if is zero vector
                v.dot(&v_path)/(v.distance_to(&ORIGIN) * v_path.distance_to(&ORIGIN)) > 0.999999 // 1-epsilon to account for floating point error                                                    
            };
            if is_colinear_to_path(&v2) && (is_colinear_to_path(&v1) ^ is_colinear_to_path(&v3)) && i < self.path_points.len() - 2 { // 2 to ensure last point is always there as well as i+1
                let mut dist = 0.0;
                let mut remove_count = 1;
                let center = self.path_points[i].position;
                while dist < self.model.config.scale*0.25 as f32 && i+remove_count < self.path_points.len() && i-remove_count > 0 {  
                    dist = self.path_points[i-remove_count].position.distance_to(&center);
                    if dist < self.model.config.scale*0.25 && i+remove_count < self.path_points.len() && i-remove_count > 0 {
                        remove_count += 1;
                    }
                }
                let start_idx = i - remove_count;
                let end_idx = (i + remove_count).min(self.path_points.len() - 1);
                for idx in start_idx..=end_idx {
                    points.push(idx);
                }
                i = end_idx + 1;
            } else {
                i += 1;
            }
        }

        println!("points: {:?}", points);
        for idx in points.into_iter().rev() {
            self.path_points.remove(idx);    
        }
    }

    pub fn follow_path(&mut self) {
        self.follow_path_with_dt(0.02); 
    }
    
    pub fn follow_path_with_dt(&mut self, dt: f32) {
        if self.current_path_progress >= 1.0 {
            self.follow_path = false;
            self.set_velocity(0.0, 0.0);
            return;
        }

        if self.velocity_update_timer >= 0.02 {
            let current_position = self.catmull_rom_spline(self.current_path_progress);
            let mut d = 0.0;
            let mut xv = 0.0;
            let mut yv = 0.0;

            let mut ci = 0.0;
            let target_distance = self.target_speed * dt;
            
            if target_distance <= 0.001 {
                ci = 0.001;
                let focus_point = self.catmull_rom_spline(self.current_path_progress + ci);
                xv = focus_point.0 - current_position.0;
                yv = focus_point.1 - current_position.1;
                d = (xv*xv + yv*yv).sqrt();
            } else {
                let max_iterations = 1000;
                let mut iteration_count = 0;
                
                while d < target_distance && iteration_count < max_iterations {
                    ci += 0.001;
                    iteration_count += 1;

                    let focus_point = self.catmull_rom_spline(self.current_path_progress + ci);
                    xv = focus_point.0 - current_position.0;
                    yv = focus_point.1 - current_position.1;
        
                    d = (xv*xv + yv*yv).sqrt();
                    
                    if iteration_count % 100 == 0 {
                        println!("Path following: progress={:.3}, distance={:.3}/{:.3}, dt={:.3}", 
                                 self.current_path_progress, d, target_distance, dt);
                    }
                    
                    if self.current_path_progress + ci >= 1.0 {
                        break;
                    }
                }
                
                if iteration_count >= max_iterations {
                    println!("Warning: Max iterations reached in follow_path calculation");
                }
            }
            
            xv /= d;
            yv /= d;

            self.set_velocity(xv * self.target_speed, yv * self.target_speed);
            
            self.current_path_progress += ci;
            if self.current_path_progress >= 1.0 {
                self.current_path_progress = 1.0;
                self.follow_path = false;
                self.set_velocity(0.0, 0.0);
            }
            
            self.velocity_update_timer = 0.0;
        }
    }
}
