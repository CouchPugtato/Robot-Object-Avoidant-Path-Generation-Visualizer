use crate::wire::{Position, Wire};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct ModelConfig {
    pub name: String,
    pub position: Position,
    pub scale: f32,
}

#[derive(Default, Debug)]
pub struct Model {
    pub wires: Vec<Wire>,
    pub config: Option<ModelConfig>,
}

impl Model {
    /// Create a model from a ModelConfig
    pub fn from_config(config: &ModelConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let file_name = if config.name.ends_with(".stl") // optional .stl
            { config.name.clone() }
            else { format!("{}.stl", config.name) };
        
        let path = Path::new("models").join(file_name);
        
        let mut model = Self::from_stl(path)?;
        
        model.scale(config.scale);
        model.position_at(config.position);

        model.config = Some(config.clone());
        
        Ok(model)
    }
    
    /// Load a model from an STL file and convert it into a wireframe
    pub fn from_stl<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);
        
        let stl = stl_io::read_stl(&mut reader)?;
        
        let mut wires = Vec::new();
        
        for face in stl.faces.iter() {
            let v1_idx = face.vertices[0];
            let v2_idx = face.vertices[1];
            let v3_idx = face.vertices[2];
            
            let v1 = Position::new(
                stl.vertices[v1_idx][0],
                stl.vertices[v1_idx][1],
                -stl.vertices[v1_idx][2], // z axis needs to be inverted
            );
            let v2 = Position::new(
                stl.vertices[v2_idx][0],
                stl.vertices[v2_idx][1],
                -stl.vertices[v2_idx][2],
            );
            let v3 = Position::new(
                stl.vertices[v3_idx][0],
                stl.vertices[v3_idx][1],
                -stl.vertices[v3_idx][2],
            );
            
            // create wires for each edge of the triangle
            wires.push(Wire { start: v1, end: v2, color: nannou::color::WHITE });
            wires.push(Wire { start: v2, end: v3, color: nannou::color::WHITE });
            wires.push(Wire { start: v3, end: v1, color: nannou::color::WHITE });
        }
        
        Self::remove_duplicate_wires(&mut wires);
        
        Ok(Model { wires, config: None })
    }
    
    fn remove_duplicate_wires(wires: &mut Vec<Wire>) {
        let mut i = 0;
        while i < wires.len() {
            let mut found_duplicate = false;
            let mut j = i + 1;
            
            while j < wires.len() { // if the wires are the same edge, remove them
                if (wires[i].start.approx_equals(&wires[j].start) && wires[i].end.approx_equals(&wires[j].end)) ||
                   (wires[i].start.approx_equals(&wires[j].end) && wires[i].end.approx_equals(&wires[j].start)) {
                    wires.swap_remove(j);
                    found_duplicate = true;
                } else {
                    j += 1;
                }
            }
            
            // do not increment i if duplicate was removed
            if !found_duplicate {
                i += 1;
            }
        }
    }
    
    /// Scale the model to fit within a given size
    pub fn scale(&mut self, size: f32) {
        // prescaled bounds of the model
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut min_z = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut max_z = f32::MIN;
        
        for wire in &self.wires {
            min_x = min_x.min(wire.start.x).min(wire.end.x);
            min_y = min_y.min(wire.start.y).min(wire.end.y);
            min_z = min_z.min(wire.start.z).min(wire.end.z);
            max_x = max_x.max(wire.start.x).max(wire.end.x);
            max_y = max_y.max(wire.start.y).max(wire.end.y);
            max_z = max_z.max(wire.start.z).max(wire.end.z);
        }
        
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        let center_z = (min_z + max_z) / 2.0;
        
        let width = max_x - min_x;
        let height = max_y - min_y;
        let depth = max_z - min_z;
        
        let max_dimension = width.max(height).max(depth);
        let scale_factor = if max_dimension > 0.0 { size / max_dimension } else { 1.0 };
        
        for wire in &mut self.wires {
            wire.start.x = (wire.start.x - center_x) * scale_factor;
            wire.start.y = (wire.start.y - center_y) * scale_factor;
            wire.start.z = (wire.start.z - center_z) * scale_factor;
            
            wire.end.x = (wire.end.x - center_x) * scale_factor;
            wire.end.y = (wire.end.y - center_y) * scale_factor;
            wire.end.z = (wire.end.z - center_z) * scale_factor;
        }
    }
    
    /// Position the model at a specific location
    pub fn position_at(&mut self, pos: Position) {
        let z_offset:f32 = if self.config.is_some() && self.config.as_ref().unwrap().position.approx_equals(&crate::wire::ORIGIN)
            { self.config.as_ref().unwrap().scale / 2.0 }
            else { 0.0 };

        for wire in &mut self.wires {
            wire.start.x += pos.x;
            wire.start.y += pos.y;
            wire.start.z += pos.z + z_offset;
            
            wire.end.x += pos.x;
            wire.end.y += pos.y;
            wire.end.z += pos.z + z_offset;
        }
    }
}