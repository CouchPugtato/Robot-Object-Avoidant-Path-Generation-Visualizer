use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};

mod model;
mod wire;
mod field;

use model::{Model, ModelConfig};
use crate::wire::Position;
use crate::field::*;

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
}



fn update(app: &App, model: &mut AppModel, update: Update) {
    let ctx = model.egui.begin_frame();
    
    // ui side panel
    egui::SidePanel::right("controls_panel")
        .default_width(200.0)
        .resizable(true)
        .show(&ctx, |ui| {
            ui.collapsing("Models", |ui| {
                ui.checkbox(&mut model.show_path, "Show Path");
                ui.checkbox(&mut model.show_points, "Show Points");
                ui.checkbox(&mut model.show_gradient_function, "Show Gradient Function");
            
                
                // new model
                ui.separator();
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
                        if let Some(config) = &loaded_model.config {
                            let is_selected = model.selected_model_index == Some(i);
                            let label = format!("Model {}: {}", i + 1, config.name);
                            
                            if ui.selectable_label(is_selected, label).clicked() {
                                model.selected_model_index = Some(i);
                            }
                        }
                    }
                    
                    if let Some(index) = model.selected_model_index {
                        if index < model.models.len() {
                            let mut current_scale = 1.0;
                            let mut current_position = Position::new(0.0, 0.0, 0.0);
                            let mut has_config = false;
                            
                            if let Some(selected_model) = model.models.get(index) {
                                if let Some(config) = &selected_model.config {
                                    current_scale = config.scale;
                                    current_position = config.position;
                                    has_config = true;
                                }
                            }
                            
                            if has_config {
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
                                        if let Some(config) = &mut selected_model.config {
                                            let old_position = config.position;
                                            config.scale = scale;
                                            
                                            selected_model.position_at(Position::new(-old_position.x, -old_position.y, -old_position.z));
                                            
                                            selected_model.scale(scale);
                                            selected_model.position_at(old_position);
                                        }
                                    }
                                    
                                    if position_changed {
                                        if let Some(config) = &mut selected_model.config {
                                            let delta_x = position.x - config.position.x;
                                            let delta_y = position.y - config.position.y;
                                            let delta_z = position.z - config.position.z;
                                        
                                            config.position = position;
                                            selected_model.position_at(Position::new(delta_x, delta_y, delta_z));
                                        }
                                    }
                                }
                                
                                if delete_clicked {
                                    model.models.remove(index);
                                    model.selected_model_index = None;
                                }
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
    
    AppModel {
        _window: window_id,
        camera_position: Position::new(0.0, -2.0, 0.0),
        direction: PI / 8.0,
        rotation_y: 0.0,
        models,
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