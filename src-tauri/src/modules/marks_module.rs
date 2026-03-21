//! Marks Module - Color highlighting with JSON interface
//! 
//! Exposed capabilities:
//! - marks.create: Create a new mark
//! - marks.update: Update an existing mark
//! - marks.delete: Delete a mark by ID
//! - marks.get: Get marks in range or all marks
//! - marks.get_at: Get marks at specific position
//! - marks.clear: Clear all marks
//! - marks.delete_by_color: Delete marks by color
//! - marks.count: Get total mark count
//! - marks.export: Export marks to JSON
//! - marks.import: Import marks from JSON

use crate::modular::{Module, ModuleInfo, Capability, ModuleError};
use crate::mark_engine::{MarkEngine, MarkColor, Mark};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;

/// Marks module implementation
pub struct MarksModule;

impl MarksModule {
    pub fn new() -> Self {
        MarksModule
    }
    
    fn capability_schemas() -> Vec<Capability> {
        vec![
            // Create
            Capability {
                name: "create".to_string(),
                description: "Create a new color mark".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "File path"},
                        "start": {"type": "integer", "minimum": 0, "description": "Start byte offset"},
                        "end": {"type": "integer", "minimum": 0, "description": "End byte offset"},
                        "color": {"type": "string", "enum": ["red", "orange", "yellow", "green", "cyan", "blue", "purple", "pink", "gray"]},
                        "label": {"type": "string", "description": "Optional label"},
                        "note": {"type": "string", "description": "Optional note"}
                    },
                    "required": ["path", "start", "end", "color"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"},
                        "start": {"type": "integer"},
                        "end": {"type": "integer"},
                        "color": {"type": "string"},
                        "label": {"type": ["string", "null"]},
                        "note": {"type": ["string", "null"]},
                        "created_at": {"type": "integer"},
                        "updated_at": {"type": "integer"}
                    }
                }),
            },
            // Update
            Capability {
                name: "update".to_string(),
                description: "Update an existing mark".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "id": {"type": "string", "description": "Mark ID to update"},
                        "updates": {
                            "type": "object",
                            "properties": {
                                "start": {"type": "integer"},
                                "end": {"type": "integer"},
                                "color": {"type": "string", "enum": ["red", "orange", "yellow", "green", "cyan", "blue", "purple", "pink", "gray"]},
                                "label": {"type": "string"},
                                "note": {"type": "string"},
                                "clear_label": {"type": "boolean"},
                                "clear_note": {"type": "boolean"}
                            }
                        }
                    },
                    "required": ["path", "id", "updates"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"},
                        "start": {"type": "integer"},
                        "end": {"type": "integer"},
                        "color": {"type": "string"},
                        "label": {"type": ["string", "null"]},
                        "note": {"type": ["string", "null"]},
                        "updated_at": {"type": "integer"}
                    }
                }),
            },
            // Delete
            Capability {
                name: "delete".to_string(),
                description: "Delete a mark by ID".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "id": {"type": "string"}
                    },
                    "required": ["path", "id"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"},
                        "deleted_id": {"type": "string"}
                    }
                }),
            },
            // Get marks
            Capability {
                name: "get".to_string(),
                description: "Get all marks or marks in range".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "start": {"type": "integer", "description": "Optional: filter by start offset"},
                        "end": {"type": "integer", "description": "Optional: filter by end offset"}
                    },
                    "required": ["path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "marks": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "id": {"type": "string"},
                                    "start": {"type": "integer"},
                                    "end": {"type": "integer"},
                                    "color": {"type": "string"},
                                    "label": {"type": ["string", "null"]},
                                    "note": {"type": ["string", "null"]},
                                    "created_at": {"type": "integer"},
                                    "updated_at": {"type": "integer"}
                                }
                            }
                        },
                        "count": {"type": "integer"}
                    }
                }),
            },
            // Get marks at position
            Capability {
                name: "get_at".to_string(),
                description: "Get marks at specific position".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "offset": {"type": "integer", "description": "Byte offset"}
                    },
                    "required": ["path", "offset"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "marks": {"type": "array"},
                        "count": {"type": "integer"}
                    }
                }),
            },
            // Clear all
            Capability {
                name: "clear".to_string(),
                description: "Clear all marks".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    },
                    "required": ["path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"},
                        "cleared_count": {"type": "integer"}
                    }
                }),
            },
            // Delete by color
            Capability {
                name: "delete_by_color".to_string(),
                description: "Delete all marks of a specific color".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "color": {"type": "string", "enum": ["red", "orange", "yellow", "green", "cyan", "blue", "purple", "pink", "gray"]}
                    },
                    "required": ["path", "color"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "deleted_count": {"type": "integer"}
                    }
                }),
            },
            // Count
            Capability {
                name: "count".to_string(),
                description: "Get total mark count".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    },
                    "required": ["path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "count": {"type": "integer"},
                        "by_color": {
                            "type": "object",
                            "additionalProperties": {"type": "integer"}
                        }
                    }
                }),
            },
            // Export
            Capability {
                name: "export".to_string(),
                description: "Export marks to JSON format".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    },
                    "required": ["path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "marks_count": {"type": "integer"},
                        "marks": {"type": "array"}
                    }
                }),
            },
            // Import
            Capability {
                name: "import".to_string(),
                description: "Import marks from JSON".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "data": {
                            "type": "object",
                            "properties": {
                                "path": {"type": "string"},
                                "marks": {"type": "array"}
                            }
                        }
                    },
                    "required": ["path", "data"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"},
                        "imported_count": {"type": "integer"}
                    }
                }),
            },
            // Get colors
            Capability {
                name: "get_colors".to_string(),
                description: "Get available colors with their hex values".to_string(),
                input_schema: serde_json::json!({"type": "object"}),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "colors": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": {"type": "string"},
                                    "hex": {"type": "string"}
                                }
                            }
                        }
                    }
                }),
            },
        ]
    }
    
    fn parse_color(color_str: &str) -> Result<MarkColor, ModuleError> {
        match color_str {
            "red" => Ok(MarkColor::Red),
            "orange" => Ok(MarkColor::Orange),
            "yellow" => Ok(MarkColor::Yellow),
            "green" => Ok(MarkColor::Green),
            "cyan" => Ok(MarkColor::Cyan),
            "blue" => Ok(MarkColor::Blue),
            "purple" => Ok(MarkColor::Purple),
            "pink" => Ok(MarkColor::Pink),
            "gray" => Ok(MarkColor::Gray),
            _ => Err(ModuleError::new("invalid_color", &format!("Unknown color: {}", color_str))),
        }
    }
    
    fn mark_to_json(mark: &Mark) -> Value {
        serde_json::json!({
            "id": mark.id,
            "start": mark.start,
            "end": mark.end,
            "color": format!("{:?}", mark.color).to_lowercase(),
            "label": mark.label,
            "note": mark.note,
            "created_at": mark.created_at,
            "updated_at": mark.updated_at,
        })
    }
}

impl Module for MarksModule {
    fn info(&self) -> ModuleInfo {
        ModuleInfo {
            name: "marks".to_string(),
            version: "1.0.0".to_string(),
            description: "Color highlighting and annotation system for files".to_string(),
            capabilities: Self::capability_schemas(),
        }
    }
    
    fn execute(&self, capability: &str, input: Value) -> Result<Value, ModuleError> {
        match capability {
            "create" => self.cmd_create(input),
            "update" => self.cmd_update(input),
            "delete" => self.cmd_delete(input),
            "get" => self.cmd_get(input),
            "get_at" => self.cmd_get_at(input),
            "clear" => self.cmd_clear(input),
            "delete_by_color" => self.cmd_delete_by_color(input),
            "count" => self.cmd_count(input),
            "export" => self.cmd_export(input),
            "import" => self.cmd_import(input),
            "get_colors" => self.cmd_get_colors(input),
            _ => Err(ModuleError::new("unknown_capability", &format!("Unknown capability: {}", capability))),
        }
    }
    
    fn get_state(&self) -> Value {
        serde_json::json!({
            "type": "marks_module",
            "loaded": true
        })
    }
    
    fn set_state(&mut self, _state: Value) -> Result<(), ModuleError> {
        Ok(())
    }
}

impl MarksModule {
    fn cmd_create(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let start = input["start"].as_u64()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'start'"))? as usize;
        let end = input["end"].as_u64()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'end'"))? as usize;
        let color_str = input["color"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'color'"))?;
        let color = Self::parse_color(color_str)?;
        let label = input["label"].as_str().map(|s| s.to_string());
        let note = input["note"].as_str().map(|s| s.to_string());
        
        let timestamp = current_timestamp();
        let mark = Mark {
            id: format!("mark_{}_{}", start, timestamp),
            start,
            end,
            color,
            label,
            note,
            created_at: timestamp,
            updated_at: timestamp,
        };
        
        let mut marks = MarkEngine::get_or_create(path);
        match marks.add_mark(mark) {
            Ok(new_mark) => Ok(Self::mark_to_json(&new_mark)),
            Err(e) => Err(ModuleError::new("create_failed", &e)),
        }
    }
    
    fn cmd_update(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let id = input["id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'id'"))?;
        
        let updates = input["updates"].clone();
        let update = crate::mark_engine::MarkUpdate {
            start: updates["start"].as_u64().map(|n| n as usize),
            end: updates["end"].as_u64().map(|n| n as usize),
            color: updates["color"].as_str().and_then(|c| {
                Self::parse_color(c).ok()
            }),
            label: updates["label"].as_str().map(|s| s.to_string()),
            note: updates["note"].as_str().map(|s| s.to_string()),
            clear_label: updates["clear_label"].as_bool().unwrap_or(false),
            clear_note: updates["clear_note"].as_bool().unwrap_or(false),
        };
        
        let mut marks = MarkEngine::get_or_create(path);
        match marks.update_mark(id, update) {
            Ok(updated) => Ok(Self::mark_to_json(&updated)),
            Err(e) => Err(ModuleError::new("update_failed", &e)),
        }
    }
    
    fn cmd_delete(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let id = input["id"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'id'"))?;
        
        let mut marks = MarkEngine::get_or_create(path);
        match marks.delete_mark(id) {
            Some(deleted) => Ok(serde_json::json!({
                "success": true,
                "deleted_id": deleted.id
            })),
            None => Err(ModuleError::new("not_found", "Mark not found")),
        }
    }
    
    fn cmd_get(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        let marks = MarkEngine::get_or_create(path);
        
        let marks_list: Vec<Value> = if let (Some(start), Some(end)) = 
            (input["start"].as_u64(), input["end"].as_u64()) {
            marks.get_marks_in_range(start as usize, end as usize)
                .iter()
                .map(|m| Self::mark_to_json(m))
                .collect()
        } else {
            marks.get_all_marks()
                .iter()
                .map(|m| Self::mark_to_json(m))
                .collect()
        };
        
        Ok(serde_json::json!({
            "marks": marks_list,
            "count": marks_list.len()
        }))
    }
    
    fn cmd_get_at(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let offset = input["offset"].as_u64()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'offset'"))? as usize;
        
        let marks = MarkEngine::get_or_create(path);
        let marks_list: Vec<Value> = marks.get_marks_at(offset)
            .iter()
            .map(|m| Self::mark_to_json(m))
            .collect();
        
        Ok(serde_json::json!({
            "marks": marks_list,
            "count": marks_list.len()
        }))
    }
    
    fn cmd_clear(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        let mut marks = MarkEngine::get_or_create(path);
        let count = marks.count();
        marks.clear_all();
        
        Ok(serde_json::json!({
            "success": true,
            "cleared_count": count
        }))
    }
    
    fn cmd_delete_by_color(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        let color_str = input["color"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'color'"))?;
        let color = Self::parse_color(color_str)?;
        
        let mut marks = MarkEngine::get_or_create(path);
        let deleted = marks.delete_by_color(color);
        
        Ok(serde_json::json!({
            "deleted_count": deleted
        }))
    }
    
    fn cmd_count(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        let marks = MarkEngine::get_or_create(path);
        let all_marks = marks.get_all_marks();
        
        let mut by_color: HashMap<String, usize> = HashMap::new();
        for mark in all_marks {
            let color_name = format!("{:?}", mark.color).to_lowercase();
            *by_color.entry(color_name).or_insert(0) += 1;
        }
        
        Ok(serde_json::json!({
            "count": all_marks.len(),
            "by_color": by_color
        }))
    }
    
    fn cmd_export(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        let marks = MarkEngine::get_or_create(path);
        let export = marks.export();
        
        Ok(serde_json::json!({
            "path": export.path,
            "marks_count": export.marks.len(),
            "marks": export.marks
        }))
    }
    
    fn cmd_import(&self, input: Value) -> Result<Value, ModuleError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'path'"))?;
        
        let marks_data = input["data"]["marks"].as_array()
            .ok_or_else(|| ModuleError::new("invalid_input", "Missing 'data.marks'"))?;
        
        let mut marks = MarkEngine::get_or_create(path);
        let mut imported = 0;
        
        for mark_value in marks_data {
            let mark: Mark = serde_json::from_value(mark_value.clone())
                .map_err(|e| ModuleError::new("parse_error", &e.to_string()))?;
            
            if marks.add_mark(mark).is_ok() {
                imported += 1;
            }
        }
        
        Ok(serde_json::json!({
            "success": true,
            "imported_count": imported
        }))
    }
    
    fn cmd_get_colors(&self, _input: Value) -> Result<Value, ModuleError> {
        let colors = vec![
            serde_json::json!({"name": "red", "hex": "#ff6b6b"}),
            serde_json::json!({"name": "orange", "hex": "#ff9f43"}),
            serde_json::json!({"name": "yellow", "hex": "#feca57"}),
            serde_json::json!({"name": "green", "hex": "#1dd1a1"}),
            serde_json::json!({"name": "cyan", "hex": "#00d2d3"}),
            serde_json::json!({"name": "blue", "hex": "#54a0ff"}),
            serde_json::json!({"name": "purple", "hex": "#5f27cd"}),
            serde_json::json!({"name": "pink", "hex": "#ff9ff3"}),
            serde_json::json!({"name": "gray", "hex": "#8395a7"}),
        ];
        
        Ok(serde_json::json!({"colors": colors}))
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Register the marks module
pub fn register() {
    let module = MarksModule::new();
    crate::modular::ModuleRegistry::register("marks", Box::new(module))
        .expect("Failed to register marks module");
}
