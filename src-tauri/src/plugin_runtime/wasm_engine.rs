//! WASM Engine using Wasmtime
//!
//! Runs WebAssembly plugins with WASI support for filesystem access.

use wasmtime::{Engine, Module, Store, Instance, Func, Memory};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::WasiCtxBuilder;
use std::path::PathBuf;
use anyhow::Error;

use crate::plugin_runtime::plugin_api::{
    PluginManifest, PluginError, EditorEvent, PluginMetadata
};

/// WASM Engine configuration
pub struct WasmEngineConfig {
    pub allow_fs: bool,
    pub allow_network: bool,
    pub allow_env: bool,
    pub plugin_dir: PathBuf,
    pub preopened_dirs: Vec<PathBuf>,
}

impl Default for WasmEngineConfig {
    fn default() -> Self {
        Self {
            allow_fs: true,
            allow_network: false,
            allow_env: false,
            plugin_dir: PathBuf::new(),
            preopened_dirs: vec![],
        }
    }
}

/// WASM runtime state - now uses WasiP1Ctx for WASIp1 support
pub struct WasmRuntimeState {
    pub wasi: WasiP1Ctx,
    pub memory: Option<Memory>,
    pub plugin_id: String,
}

/// WASM Plugin Engine
pub struct WasmEngine {
    engine: Engine,
    config: WasmEngineConfig,
}

/// Loaded WASM plugin instance
pub struct WasmPlugin {
    pub manifest: PluginManifest,
    pub instance: Instance,
    pub store: Store<WasmRuntimeState>,
    pub memory: Memory,
    pub metadata: PluginMetadata,
}

impl WasmEngine {
    /// Create a new WASM engine
    pub fn new(config: WasmEngineConfig) -> Result<Self, PluginError> {
        let engine = Engine::default();
        
        Ok(Self { engine, config })
    }
    
    /// Load a WASM plugin from file
    pub fn load_plugin(
        &self,
        manifest: PluginManifest,
        wasm_path: PathBuf
    ) -> Result<WasmPlugin, PluginError> {
        // Read WASM file
        let wasm_bytes = std::fs::read(&wasm_path)
            .map_err(|e| PluginError::new("load_failed", &format!("Failed to read WASM file: {}", e)))?;
        
        // Compile module
        let module = Module::new(&self.engine, &wasm_bytes)
            .map_err(|e| PluginError::new("compile_failed", &format!("Failed to compile WASM module: {}", e)))?;
        
        // Create WASI context
        let wasi_ctx = self.build_wasi_context()
            .map_err(|e| PluginError::new("wasi_init", &format!("Failed to build WASI context: {}", e)))?;
        
        // Create store
        let mut store = Store::new(
            &self.engine,
            WasmRuntimeState {
                wasi: wasi_ctx,
                memory: None,
                plugin_id: manifest.id.clone(),
            }
        );
        
        // Create linker and add WASI imports
        let mut linker = wasmtime::Linker::new(&self.engine);
        
        // Add WASI imports using the wasmtime 23 preview1 API
        preview1::add_to_linker_sync(&mut linker, |state: &mut WasmRuntimeState| &mut state.wasi)
            .map_err(|e| PluginError::new("link_failed", &format!("Failed to add WASI imports: {}", e)))?;
        
        // Add host functions to linker
        self.add_host_functions(&mut linker, &mut store)?;
        
        // Instantiate the module
        let instance = linker.instantiate(&mut store, &module)
            .map_err(|e| PluginError::new("instantiate_failed", &format!("Failed to instantiate module: {}", e)))?;
        
        // Get memory export
        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| PluginError::new("no_memory", "Plugin does not export 'memory'"))?;
        
        // Store memory reference
        store.data_mut().memory = Some(memory);
        
        let metadata = PluginMetadata {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            description: manifest.description.clone(),
            author: manifest.author.clone(),
            loaded: true,
            enabled: true,
        };
        
        let mut plugin = WasmPlugin {
            manifest,
            instance,
            store,
            memory,
            metadata,
        };
        
        // Initialize the plugin
        plugin.initialize()?;
        
        Ok(plugin)
    }
    
    /// Build WASI context with appropriate permissions
    fn build_wasi_context(&self) -> Result<WasiP1Ctx, Box<dyn std::error::Error>> {
        let mut builder = WasiCtxBuilder::new();
        
        // Set stdin/stdout/stderr to inherit from parent
        builder.inherit_stdio();
        
        if self.config.allow_fs {
            // In wasmtime 23 preview1, we use build_p1() for core wasm modules
            builder.preopened_dir(&self.config.plugin_dir, ".", wasmtime_wasi::DirPerms::all(), wasmtime_wasi::FilePerms::all())?;
            
            // Preopen additional directories
            for dir in &self.config.preopened_dirs {
                let name = dir.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("dir");
                builder.preopened_dir(dir, name, wasmtime_wasi::DirPerms::all(), wasmtime_wasi::FilePerms::all())?;
            }
        }
        
        if self.config.allow_network {
            builder.inherit_network();
        }
        
        if self.config.allow_env {
            builder.inherit_env();
        }
        
        // Build as WASIp1 context for core wasm modules
        Ok(builder.build_p1())
    }
    
    /// Add host functions that plugins can call
    fn add_host_functions(
        &self,
        linker: &mut wasmtime::Linker<WasmRuntimeState>,
        store: &mut Store<WasmRuntimeState>,
    ) -> Result<(), PluginError> {
        // Log function: sak_log(ptr: i32, len: i32) -> ()
        let log_func = Func::wrap(&mut *store,
            |_caller: wasmtime::Caller<'_, WasmRuntimeState>, ptr: i32, len: i32| {
                log::debug!("Plugin log called: ptr={}, len={}", ptr, len);
                Ok::<(), Error>(())
            }
        );
        linker.define(&mut *store, "env", "sak_log", log_func)
            .map_err(|e| PluginError::new("link_failed", &format!("Failed to define sak_log: {}", e)))?;
        
        // Get editor content: sak_get_content(path_ptr: i32, path_len: i32, out_ptr: i32, out_len: i32) -> i32
        let get_content = Func::wrap(&mut *store,
            |_caller: wasmtime::Caller<'_, WasmRuntimeState>,
             _path_ptr: i32, _path_len: i32, _out_ptr: i32, _out_len: i32| -> Result<i32, Error> {
                log::debug!("Plugin get_content called");
                Ok::<i32, Error>(0i32) // Return bytes written
            }
        );
        linker.define(&mut *store, "env", "sak_get_content", get_content)
            .map_err(|e| PluginError::new("link_failed", &format!("Failed to define sak_get_content: {}", e)))?;
        
        // Set editor content: sak_set_content(path_ptr: i32, path_len: i32, content_ptr: i32, content_len: i32) -> i32
        let set_content = Func::wrap(&mut *store,
            |_caller: wasmtime::Caller<'_, WasmRuntimeState>,
             _path_ptr: i32, _path_len: i32, _content_ptr: i32, _content_len: i32| -> Result<i32, Error> {
                log::debug!("Plugin set_content called");
                Ok::<i32, Error>(0i32) // Return success
            }
        );
        linker.define(&mut *store, "env", "sak_set_content", set_content)
            .map_err(|e| PluginError::new("link_failed", &format!("Failed to define sak_set_content: {}", e)))?;
        
        // Show notification: sak_show_notification(title_ptr: i32, title_len: i32, msg_ptr: i32, msg_len: i32) -> ()
        let show_notification = Func::wrap(&mut *store,
            |_caller: wasmtime::Caller<'_, WasmRuntimeState>,
             _title_ptr: i32, _title_len: i32, _msg_ptr: i32, _msg_len: i32| -> Result<(), Error> {
                log::debug!("Plugin show_notification called");
                Ok::<(), Error>(())
            }
        );
        linker.define(&mut *store, "env", "sak_show_notification", show_notification)
            .map_err(|e| PluginError::new("link_failed", &format!("Failed to define sak_show_notification: {}", e)))?;
        
        // Get plugin setting: sak_get_setting(key_ptr: i32, key_len: i32, out_ptr: i32, out_len: i32) -> i32
        let get_setting = Func::wrap(&mut *store,
            |_caller: wasmtime::Caller<'_, WasmRuntimeState>,
             _key_ptr: i32, _key_len: i32, _out_ptr: i32, _out_len: i32| -> Result<i32, Error> {
                log::debug!("Plugin get_setting called");
                Ok::<i32, Error>(0i32)
            }
        );
        linker.define(&mut *store, "env", "sak_get_setting", get_setting)
            .map_err(|e| PluginError::new("link_failed", &format!("Failed to define sak_get_setting: {}", e)))?;
        
        // Set plugin setting: sak_set_setting(key_ptr: i32, key_len: i32, value_ptr: i32, value_len: i32) -> i32
        let set_setting = Func::wrap(&mut *store,
            |_caller: wasmtime::Caller<'_, WasmRuntimeState>,
             _key_ptr: i32, _key_len: i32, _value_ptr: i32, _value_len: i32| -> Result<i32, Error> {
                log::debug!("Plugin set_setting called");
                Ok::<i32, Error>(0i32)
            }
        );
        linker.define(&mut *store, "env", "sak_set_setting", set_setting)
            .map_err(|e| PluginError::new("link_failed", &format!("Failed to define sak_set_setting: {}", e)))?;
        
        Ok(())
    }
}

impl WasmPlugin {
    /// Initialize the plugin
    fn initialize(&mut self) -> Result<(), PluginError> {
        // Look for __initialize or _initialize export
        if let Some(init) = self.instance.get_func(&mut self.store, "__initialize") {
            init.typed::<(), ()>(&mut self.store)
                .map_err(|e| PluginError::new("init_failed", &format!("Failed to get init signature: {}", e)))?
                .call(&mut self.store, ())
                .map_err(|e| PluginError::new("init_failed", &format!("Failed to call initialize: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Execute a capability by name
    pub fn execute_capability(&mut self, capability_id: &str, input: &str) -> Result<String, PluginError> {
        // Look for exported function matching capability_id
        let func_name = format!("__capability_{}", capability_id.replace("-", "_"));
        
        if let Some(func) = self.instance.get_func(&mut self.store, &func_name) {
            // Write input to memory
            let input_bytes = input.as_bytes();
            let input_ptr = self.allocate_memory(input_bytes.len())?;
            self.write_memory(input_ptr, input_bytes)?;
            
            // Call function
            // TODO: Handle return values properly
            func.typed::<(i32, i32), i32>(&mut self.store)
                .map_err(|e| PluginError::new("exec_failed", &format!("Failed to get function type: {}", e)))?
                .call(&mut self.store, (input_ptr as i32, input_bytes.len() as i32))
                .map_err(|e| PluginError::new("exec_failed", &format!("Function call failed: {}", e)))?;
            
            // Free allocated memory
            self.free_memory(input_ptr, input_bytes.len())?;
            
            Ok("{}".to_string()) // Return empty JSON for now
        } else {
            Err(PluginError::new("capability_not_found", &format!("Capability '{}' not found", capability_id)))
        }
    }
    
    /// Handle editor event
    pub fn on_event(&mut self, event: &EditorEvent) -> Result<(), PluginError> {
        if let Some(handler) = self.instance.get_func(&mut self.store, "__on_event") {
            // Serialize event to JSON
            let event_json = serde_json::to_string(event)
                .map_err(|e| PluginError::new("event_serialize", &e.to_string()))?;
            
            // Write to memory and call handler
            let event_bytes = event_json.as_bytes();
            let event_ptr = self.allocate_memory(event_bytes.len())?;
            self.write_memory(event_ptr, event_bytes)?;
            
            handler.typed::<(i32, i32), ()>(&mut self.store)
                .map_err(|e| PluginError::new("event_handler", &format!("Failed to get event handler type: {}", e)))?
                .call(&mut self.store, (event_ptr as i32, event_bytes.len() as i32))
                .map_err(|e| PluginError::new("event_handler", &format!("Event handler failed: {}", e)))?;
            
            self.free_memory(event_ptr, event_bytes.len())?;
        }
        
        Ok(())
    }
    
    /// Shutdown the plugin
    pub fn shutdown(&mut self) -> Result<(), PluginError> {
        if let Some(shutdown) = self.instance.get_func(&mut self.store, "__shutdown") {
            shutdown.typed::<(), ()>(&mut self.store)
                .map_err(|e| PluginError::new("shutdown_failed", &format!("Failed to get shutdown signature: {}", e)))?
                .call(&mut self.store, ())
                .map_err(|e| PluginError::new("shutdown_failed", &format!("Shutdown failed: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Allocate memory in WASM linear memory
    fn allocate_memory(&mut self, size: usize) -> Result<usize, PluginError> {
        // Try to call exported alloc function
        if let Some(alloc) = self.instance.get_func(&mut self.store, "__alloc") {
            let result = alloc.typed::<i32, i32>(&mut self.store)
                .map_err(|e| PluginError::new("alloc_failed", &format!("Failed to get alloc signature: {}", e)))?
                .call(&mut self.store, size as i32)
                .map_err(|e| PluginError::new("alloc_failed", &format!("Allocation failed: {}", e)))?;
            
            Ok(result as usize)
        } else {
            // Fallback: just return 0 and let caller handle it
            // In a real implementation, we'd need a proper allocator
            Ok(0)
        }
    }
    
    /// Free allocated memory
    fn free_memory(&mut self, _ptr: usize, _size: usize) -> Result<(), PluginError> {
        if let Some(dealloc) = self.instance.get_func(&mut self.store, "__dealloc") {
            dealloc.typed::<(i32, i32), ()>(&mut self.store)
                .map_err(|e| PluginError::new("dealloc_failed", &format!("Failed to get dealloc signature: {}", e)))?
                .call(&mut self.store, (_ptr as i32, _size as i32))
                .map_err(|e| PluginError::new("dealloc_failed", &format!("Deallocation failed: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Write data to WASM memory
    fn write_memory(&mut self, offset: usize, data: &[u8]) -> Result<(), PluginError> {
        let mem = self.instance.get_memory(&mut self.store, "memory")
            .ok_or_else(|| PluginError::new("no_memory", "Memory export not found"))?;
        
        mem.write(&mut self.store, offset, data)
            .map_err(|e| PluginError::new("write_failed", &format!("Failed to write to memory: {}", e)))?;
        
        Ok(())
    }
    
    /// Read data from WASM memory
    #[allow(dead_code)]
    fn read_memory(&mut self, offset: usize, len: usize) -> Result<Vec<u8>, PluginError> {
        let mem = self.instance.get_memory(&mut self.store, "memory")
            .ok_or_else(|| PluginError::new("no_memory", "Memory export not found"))?;
        
        let mut buffer = vec![0u8; len];
        mem.read(&mut self.store, offset, &mut buffer)
            .map_err(|e| PluginError::new("read_failed", &format!("Failed to read from memory: {}", e)))?;
        
        Ok(buffer)
    }
}
