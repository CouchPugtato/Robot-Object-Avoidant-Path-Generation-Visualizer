use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};

mod model;
mod wire;
mod field;
mod obstacle;
mod position;
mod gradient_field;

use model::{Model, ModelConfig};
use crate::position::Position;
use crate::field::*;
use crate::obstacle::Obstacle;

const SPEED: f64 = 3.0;
const FOV: f32 = PI / 2.0;
const SCREENWIDTH: u32 = 640;
const SCREENHEIGHT: u32 = 480;

fn main() {
    nannou::app(model)
        .update(update)
        .run();
}

struct AppModel {
    _window: window::Id,
    camera_position: Position,
    direction: f32,
    rotation_y: f32,
    models: Vec<Model>,
    obstacles: Vec<Obstacle>,
    egui: Egui,

    camera_speed: f32,
    rotation_speed: f32,
    show_path: bool,
    show_points: bool,
    show_gradient_function: bool,

    new_model_name: String,
    new_model_scale: f32,
    new_model_position: Position,
    selected_model_index: Option<usize>,
    selected_obstacle_index: Option<usize>,
    create_as_obstacle: bool,
    
    new_obstacle_name: String,
    new_obstacle_radius: f32,
    new_obstacle_position: Position,
    
    gradient_field: Option<gradient_field::GradientWire>,
    gradient_x_resolution: f32,
    gradient_y_resolution: f32,
    gradient_line_resolution: f32,
}



fn update(app: &App, model: &mut AppModel, update: Update) {
    let ctx = model.egui.begin_frame();
    
    // ui side panel
    egui::SidePanel::right("controls_panel")
        .default_width(200.0)
        .resizable(true)
        .show(&ctx, |ui| {
            ui.collapsing("Visualization", |ui| {
                ui.checkbox(&mut model.show_path, "Show Path");
                ui.checkbox(&mut model.show_points, "Show Points");
                ui.checkbox(&mut model.show_gradient_function, "Show Gradient Function");
                
                if model.show_gradient_function {
                    ui.separator();
                    ui.heading("Gradient Field Settings");
                                        
                    ui.add(egui::Slider::new(&mut model.gradient_x_resolution, 0.01..=10.0).text("X Resolution"));
                    ui.add(egui::Slider::new(&mut model.gradient_y_resolution, 0.01..=10.0).text("Y Resolution"));
                    ui.add(egui::Slider::new(&mut model.gradient_line_resolution, 0.01..=1.0).text("Line Resolution"));
                    
                    if ui.button("Update Gradient Field").clicked() {
                        let gradient_function = gradient_field::obstacle_sum;
                        
                        gradient_field::set_obstacles_ref(&model.obstacles);
                        
                        if let Some(gradient_field) = &mut model.gradient_field {
                            gradient_field.gradient_function = gradient_function;
                            gradient_field.x_resolution = model.gradient_x_resolution;
                            gradient_field.y_resolution = model.gradient_y_resolution;
                            gradient_field.line_resolution = model.gradient_line_resolution;
                            gradient_field.update();
                        } else {
                            gradient_field::set_obstacles_ref(&model.obstacles);
                            
                            model.gradient_field = Some(gradient_field::GradientWire::new(
                                gradient_function,
                                model.gradient_x_resolution,
                                model.gradient_y_resolution,
                                model.gradient_line_resolution
                            ).with_color(nannou::color::rgb(0, 255, 255)));
                        }
                    }
                }
            });
            
            ui.collapsing("Models", |ui| {
                // new model
                ui.heading("Create New Model");
                
                ui.horizontal(|ui| {
                    ui.label("Model Name:");
                    ui.text_edit_singleline(&mut model.new_model_name);
                });
                
                ui.add(egui::Slider::new(&mut model.new_model_scale, 0.1..=10.0).text("Scale"));
                
                // model position controls
                ui.label("Position:");
                ui.horizontal(|ui| {
                    ui.label("X:");
                    ui.add(egui::DragValue::new(&mut model.new_model_position.x).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Y:");
                    ui.add(egui::DragValue::new(&mut model.new_model_position.y).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Z:");
                    ui.add(egui::DragValue::new(&mut model.new_model_position.z).speed(0.1));
                });
                
                if ui.button("Add Model").clicked() {
                    let config = ModelConfig {
                        name: model.new_model_name.clone(),
                        position: model.new_model_position,
                        scale: model.new_model_scale
                    };
                    
                    match Model::from_config(&config) {
                        Ok(loaded_model) => {
                            model.models.push(loaded_model);
                            println!("Successfully loaded model: {}", config.name);
                            model.selected_model_index = Some(model.models.len() - 1);
                        },
                        Err(e) => { eprintln!("Failed to load model {}: {}", config.name, e); }
                    }
                }
                
                if !model.models.is_empty() {
                    ui.separator();
                    ui.heading("Existing Models");
                    
                    for (i, loaded_model) in model.models.iter_mut().enumerate() {
                        let is_selected = model.selected_model_index == Some(i);
                        let label = format!("Model {}: {}", i + 1, loaded_model.config.name);
                        
                        if ui.selectable_label(is_selected, label).clicked() {
                            model.selected_model_index = Some(i);
                            model.selected_obstacle_index = None;
                        }
                    }
                }
                
                // edit selected model
                if let Some(index) = model.selected_model_index {
                    if index < model.models.len() {
                        let mut current_scale = 1.0;
                        let mut current_position = Position::new(0.0, 0.0, 0.0);
                        
                        if let Some(selected_model) = model.models.get(index) {
                            current_scale = selected_model.config.scale;
                            current_position = selected_model.config.position;
                        }
                        
                        ui.separator();
                        ui.heading("Edit Selected Model");
                        
                        // scale control
                        let mut scale = current_scale;
                        let scale_changed = ui.add(egui::Slider::new(&mut scale, 0.1..=10.0).text("Scale")).changed();
                        
                        // position controls
                        ui.label("Position:");
                        let mut position = current_position;
                        let mut position_changed = false;
                        
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            if ui.add(egui::DragValue::new(&mut position.x).speed(0.1)).changed() {
                                position_changed = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Y:");
                            if ui.add(egui::DragValue::new(&mut position.y).speed(0.1)).changed() {
                                position_changed = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Z:");
                            if ui.add(egui::DragValue::new(&mut position.z).speed(0.1)).changed() {
                                position_changed = true;
                            }
                        });
                        
                        let delete_clicked = ui.button("Delete Model").clicked();
                        
                        if let Some(selected_model) = model.models.get_mut(index) {
                            if scale_changed {
                                let old_position = selected_model.config.position;
                                selected_model.config.scale = scale;
                                
                                selected_model.position_at(Position::new(-old_position.x, -old_position.y, -old_position.z));
                                
                                selected_model.scale(scale);
                                selected_model.position_at(old_position);
                            }
                            
                            if position_changed {
                                let delta_x = position.x - selected_model.config.position.x;
                                let delta_y = position.y - selected_model.config.position.y;
                                let delta_z = position.z - selected_model.config.position.z;
                            
                                selected_model.config.position = position;
                                selected_model.position_at(Position::new(delta_x, delta_y, delta_z));
                            }
                        }
                        
                        if delete_clicked {
                            model.models.remove(index);
                            model.selected_model_index = None;
                        }
                    }
                }
            });
            
            // obstacles section
            ui.collapsing("Obstacles", |ui| {
                ui.heading("Create New Obstacle");
                
                ui.horizontal(|ui| {
                    ui.label("Obstacle Name:");
                    ui.text_edit_singleline(&mut model.new_obstacle_name);
                });
                
                ui.add(egui::Slider::new(&mut model.new_obstacle_radius, 0.1..=5.0).text("Radius"));
                
                // obstacle position controls
                ui.label("Position:");
                ui.horizontal(|ui| {
                    ui.label("X:");
                    ui.add(egui::DragValue::new(&mut model.new_obstacle_position.x).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Y:");
                    ui.add(egui::DragValue::new(&mut model.new_obstacle_position.y).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Z:");
                    ui.add(egui::DragValue::new(&mut model.new_obstacle_position.z).speed(0.1));
                });
                
                if ui.button("Add Obstacle").clicked() {
                    let config = ModelConfig {
                        name: model.new_obstacle_name.clone(),
                        position: model.new_obstacle_position,
                        scale: model.new_obstacle_radius * 2.0 // scale is diameter, radius*2
                    };
                    
                    match Obstacle::from_config(&config) {
                        Ok(obstacle) => {
                            model.obstacles.push(obstacle);
                            println!("Successfully created obstacle: {}", config.name);
                            model.selected_obstacle_index = Some(model.obstacles.len() - 1);
                            model.selected_model_index = None;
                            
                            if model.show_gradient_function {
                                gradient_field::set_obstacles_ref(&model.obstacles);
                                
                                if let Some(gradient_field) = &mut model.gradient_field {
                                    gradient_field.update();
                                }
                            }
                        },
                        Err(e) => { eprintln!("Failed to create obstacle {}: {}", config.name, e); }
                    }
                }
                
                if !model.obstacles.is_empty() {
                    ui.separator();
                    ui.heading("Existing Obstacles");
                    
                    for (i, obstacle) in model.obstacles.iter().enumerate() {
                        let is_selected = model.selected_obstacle_index == Some(i);
                        let label = format!("Obstacle {}: {} (radius: {:.2})", 
                            i + 1, 
                            obstacle.model.config.name,
                            obstacle.get_radius()
                        );
                        
                        if ui.selectable_label(is_selected, label).clicked() {
                            model.selected_obstacle_index = Some(i);
                            model.selected_model_index = None;
                        }
                    }
                }
                
                if let Some(index) = model.selected_obstacle_index {
                    if index < model.obstacles.len() {
                        let mut current_radius = 1.0;
                        let mut current_position = Position::new(0.0, 0.0, 0.0);
                        
                        if let Some(selected_obstacle) = model.obstacles.get(index) {
                            current_radius = selected_obstacle.get_radius();
                            current_position = selected_obstacle.model.config.position;
                        }
                        
                        ui.separator();
                        ui.heading("Edit Selected Obstacle");
                        
                        // radius control
                        let mut radius = current_radius;
                        let radius_changed = ui.add(egui::Slider::new(&mut radius, 0.1..=5.0).text("Radius")).changed();
                        
                        // position controls
                        ui.label("Position:");
                        let mut position = current_position;
                        let mut position_changed = false;
                        
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            if ui.add(egui::DragValue::new(&mut position.x).speed(0.1)).changed() {
                                position_changed = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Y:");
                            if ui.add(egui::DragValue::new(&mut position.y).speed(0.1)).changed() {
                                position_changed = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Z:");
                            if ui.add(egui::DragValue::new(&mut position.z).speed(0.1)).changed() {
                                position_changed = true;
                            }
                        });
                        
                        let delete_clicked = ui.button("Delete Obstacle").clicked();
                        
                        if let Some(selected_obstacle) = model.obstacles.get_mut(index) {
                            if radius_changed {
                                selected_obstacle.set_radius(radius);
                                
                                // update the model scale to match the new radius (diameter = 2*radius)
                                let old_position = selected_obstacle.model.config.position;
                                selected_obstacle.model.config.scale = radius * 2.0;
                                
                                selected_obstacle.model.position_at(Position::new(-old_position.x, -old_position.y, -old_position.z));
                                selected_obstacle.model.scale(radius * 2.0);
                                selected_obstacle.model.position_at(old_position);
                            }
                            
                            if position_changed {
                                let delta_x = position.x - selected_obstacle.model.config.position.x;
                                let delta_y = position.y - selected_obstacle.model.config.position.y;
                                let delta_z = position.z - selected_obstacle.model.config.position.z;
                            
                                selected_obstacle.model.config.position = position;
                                selected_obstacle.model.position_at(Position::new(delta_x, delta_y, delta_z));
                                
                                position_changed = true;
                            }
                        }
                        
                        if delete_clicked {
                            model.obstacles.remove(index);
                            model.selected_obstacle_index = None;
                        }
                        
                        if (radius_changed || position_changed || delete_clicked) && model.show_gradient_function {
                            gradient_field::set_obstacles_ref(&model.obstacles);
                            
                            if let Some(gradient_field) = &mut model.gradient_field {
                                gradient_field.update();
                            }
                        }
                    }
                }
            });
            
            ui.collapsing("Speed Settings", |ui| {
                ui.add(egui::Slider::new(&mut model.camera_speed, 0.5..=10.0).text("Camera Speed"));
                ui.add(egui::Slider::new(&mut model.rotation_speed, 0.1..=5.0).text("Rotation Speed"));
            });
            
            ui.separator();
            ui.label("Controls:");
            ui.label("WASD - Move Camera");
            ui.label("E - Up, Q - Down");
            ui.label("Arrow Keys - Rotate Camera");
        });
    
    let step_size = (update.since_last.secs() * model.camera_speed as f64) as f32;

    if app.keys.down.contains(&Key::W) {
        model.camera_position.move_by(
            step_size * model.direction.cos(),
            step_size * model.direction.sin(),
            0.0,
        );
    }
    if app.keys.down.contains(&Key::S) {
        model.camera_position.move_by(
            -step_size * model.direction.cos(),
            -step_size * model.direction.sin(),
            0.0,
        );
    }
    if app.keys.down.contains(&Key::A) {
        model.camera_position.move_by(
            -step_size * model.direction.sin(),
            step_size * model.direction.cos(),
            0.0,
        );
    }
    if app.keys.down.contains(&Key::D) {
        model.camera_position.move_by(
            step_size * model.direction.sin(),
            -step_size * model.direction.cos(),
            0.0,
        );
    }

    if app.keys.down.contains(&Key::E) {
        model.camera_position.move_by(
            0.0,
            0.0,
            step_size,
        );
    }
    if app.keys.down.contains(&Key::Q) {
        model.camera_position.move_by(
            0.0,
            0.0,
            -step_size,
        );
    }

    let rot_step = (update.since_last.secs() * model.rotation_speed as f64) as f32;
    let rot_y_step = (update.since_last.secs() * model.rotation_speed as f64) as f32;

    if app.keys.down.contains(&Key::Left) { model.direction += rot_step; }
    if app.keys.down.contains(&Key::Right) { model.direction -= rot_step; }
    if app.keys.down.contains(&Key::Up) { model.rotation_y += rot_y_step; }
    if app.keys.down.contains(&Key::Down) { model.rotation_y -= rot_y_step; }
}

fn model(app: &App) -> AppModel {
    let window_id = app
        .new_window()
        .size(SCREENWIDTH + 250, SCREENHEIGHT)
        .view(view)
        .raw_event(|_app: &App, model: &mut AppModel, event: &nannou::winit::event::WindowEvent| model.egui.handle_raw_event(event))
        .build()
        .unwrap();
    
    let window = app.window(window_id).unwrap();
    
    let egui = Egui::from_window(&window);
    
    let models = Vec::new();
    let obstacles = Vec::new();
    
    gradient_field::set_obstacles_ref(&obstacles);
    
    let gradient_field = Some(gradient_field::GradientWire::new(
        gradient_field::obstacle_sum,
        0.5,
        0.5,
        0.5
    ).with_color(nannou::color::rgb(0, 255, 255)));
    
    AppModel {
        _window: window_id,
        camera_position: Position::new(0.0, -2.0, 0.0),
        direction: PI / 8.0,
        rotation_y: 0.0,
        models,
        obstacles,
        egui,
        camera_speed: SPEED as f32,
        rotation_speed: 1.0,
        show_path: true,
        show_points: true,
        show_gradient_function: true,
        new_model_name: String::from("cube"),
        new_model_scale: 1.0, // default
        new_model_position: Position::new(0.0, 0.0, 0.0),
        selected_model_index: None,
        selected_obstacle_index: None,
        create_as_obstacle: false,
        
        new_obstacle_name: String::from("robot_base"),
        new_obstacle_radius: 1.0,
        new_obstacle_position: Position::new(0.0, 0.0, 0.0),
        
        gradient_field,
        gradient_x_resolution: 0.5,
        gradient_y_resolution: 0.5,
        gradient_line_resolution: 0.5,
    }
}

fn view(app: &App, model: &AppModel, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    
    for wire in field_border() {
        let cam_pos_start: Position = to_cam_coords(
            wire.start,
            model.camera_position,
            model.direction,
            model.rotation_y,
        );
        let cam_pos_end: Position = to_cam_coords(
            wire.end,
            model.camera_position,
            model.direction,
            model.rotation_y,
        );
        let draw_start: Vec2 = point_on_canvas(cam_pos_start);
        let draw_end: Vec2 = point_on_canvas(cam_pos_end);
        draw.line().start(draw_start).end(draw_end).color(wire.color);
    }
    
    if model.show_gradient_function {
        if let Some(gradient_field) = &model.gradient_field {
            for wire in gradient_field.get_all_wires() {
                let cam_pos_start: Position = to_cam_coords(
                    wire.start,
                    model.camera_position,
                    model.direction,
                    model.rotation_y,
                );
                let cam_pos_end: Position = to_cam_coords(
                    wire.end,
                    model.camera_position,
                    model.direction,
                    model.rotation_y,
                );
                let draw_start: Vec2 = point_on_canvas(cam_pos_start);
                let draw_end: Vec2 = point_on_canvas(cam_pos_end);
                draw.line().start(draw_start).end(draw_end).color(wire.color);
            }
        }
    }
    
    for loaded_model in &model.models {
        for wire in &loaded_model.wires {
            let cam_pos_start: Position = to_cam_coords(
                wire.start,
                model.camera_position,
                model.direction,
                model.rotation_y,
            );
            let cam_pos_end: Position = to_cam_coords(
                wire.end,
                model.camera_position,
                model.direction,
                model.rotation_y,
            );
            let draw_start: Vec2 = point_on_canvas(cam_pos_start);
            let draw_end: Vec2 = point_on_canvas(cam_pos_end);
            draw.line().start(draw_start).end(draw_end).color(wire.color);
        }
    }
    
    for obstacle in &model.obstacles {
        for wire in &obstacle.wires {
            let cam_pos_start: Position = to_cam_coords(
                wire.start,
                model.camera_position,
                model.direction,
                model.rotation_y,
            );
            let cam_pos_end: Position = to_cam_coords(
                wire.end,
                model.camera_position,
                model.direction,
                model.rotation_y,
            );
            let draw_start: Vec2 = point_on_canvas(cam_pos_start);
            let draw_end: Vec2 = point_on_canvas(cam_pos_end);
            draw.line().start(draw_start).end(draw_end).color(GREEN);
        }
    }
    
    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}

fn to_cam_coords(pos: Position, cam: Position, direction: f32, rotation_y: f32) -> Position {
    let mut r_pos: Position = Position::new(pos.x - cam.x, pos.y - cam.y, -(pos.z - cam.z));

    let mut rx: f32 = r_pos.x;
    let ry: f32 = r_pos.y;

    r_pos.x = rx * (-direction).cos() - ry * (-direction).sin();
    r_pos.y = rx * (-direction).sin() + ry * (-direction).cos();

    rx = r_pos.x;
    let rz: f32 = r_pos.z;

    r_pos.x = rx * (-rotation_y).cos() + rz * (-rotation_y).sin();
    r_pos.z = rz * (-rotation_y).cos() - rx * (-rotation_y).sin();

    r_pos
}

fn point_on_canvas(pos: Position) -> Vec2 {
    let mut angle_h = pos.y.atan2(pos.x);
    let mut angle_v = pos.z.atan2(pos.x);

    // remove fishbowl effect
    angle_h /= angle_h.cos().abs();
    angle_v /= angle_v.cos().abs();

    vec2(
        -angle_h * SCREENWIDTH as f32 / FOV,
        -angle_v * SCREENHEIGHT as f32 / FOV,
    )
}