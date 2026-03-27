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
        log::info!("[plugin:wasm_engine] Creating new WASM engine");
        log::debug!("[plugin:wasm_engine] Engine config: allow_fs={}, allow_network={}, allow_env={}",
            config.allow_fs, config.allow_network, config.allow_env);
        log::debug!("[plugin:wasm_engine] Plugin directory: {:?}", config.plugin_dir);
        
        let engine = Engine::default();
        log::debug!("[plugin:wasm_engine] Wasmtime Engine created with default configuration");
        
        log::info!("[plugin:wasm_engine] WASM engine created successfully");
        Ok(Self { engine, config })
    }
    
    /// Load a WASM plugin from file
    pub fn load_plugin(
        &self,
        manifest: PluginManifest,
        wasm_path: PathBuf
    ) -> Result<WasmPlugin, PluginError> {
        log::info!("[plugin:wasm_engine] Loading WASM plugin: {} ({}) from {:?}",
            manifest.name, manifest.id, wasm_path);
        log::debug!("[plugin:wasm_engine] Plugin has {} capabilities: {:?}",
            manifest.capabilities.len(), 
            manifest.capabilities.iter().map(|c| &c.id).collect::<Vec<_>>());
        log::debug!("[plugin:wasm_engine] Plugin has {} permissions: {:?}",
            manifest.permissions.len(), manifest.permissions);
        
        // Read WASM file
        log::debug!("[plugin:wasm_engine] Reading WASM file: {:?}", wasm_path);
        let wasm_bytes = std::fs::read(&wasm_path)
            .map_err(|e| {
                log::error!("[plugin:wasm_engine] Failed to read WASM file {:?}: {}", wasm_path, e);
                PluginError::new("load_failed", &format!("Failed to read WASM file: {}", e))
            })?;
        log::info!("[plugin:wasm_engine] WASM file loaded: {} bytes", wasm_bytes.len());
        
        // Compile module
        log::debug!("[plugin:wasm_engine] Compiling WASM module...");
        let module = Module::new(&self.engine, &wasm_bytes)
            .map_err(|e| {
                log::error!("[plugin:wasm_engine] Failed to compile WASM module: {}", e);
                PluginError::new("compile_failed", &format!("Failed to compile WASM module: {}", e))
            })?;
        log::info!("[plugin:wasm_engine] WASM module compiled successfully");
        log::debug!("[plugin:wasm_engine] Module imports: {:?}", module.imports().map(|i| i.name()).collect::<Vec<_>>());
        log::debug!("[plugin:wasm_engine] Module exports: {:?}", module.exports().map(|e| e.name()).collect::<Vec<_>>());
        
        // Create WASI context
        log::debug!("[plugin:wasm_engine] Building WASI context...");
        let wasi_ctx = self.build_wasi_context()
            .map_err(|e| {
                log::error!("[plugin:wasm_engine] Failed to build WASI context: {}", e);
                PluginError::new("wasi_init", &format!("Failed to build WASI context: {}", e))
            })?;
        log::info!("[plugin:wasm_engine] WASI context created successfully");
        
        // Create store
        log::debug!("[plugin:wasm_engine] Creating Wasmtime Store...");
        let mut store = Store::new(
            &self.engine,
            WasmRuntimeState {
                wasi: wasi_ctx,
                memory: None,
                plugin_id: manifest.id.clone(),
            }
        );
        log::info!("[plugin:wasm_engine] Wasmtime Store created");
        
        // Create linker and add WASI imports
        log::debug!("[plugin:wasm_engine] Creating linker...");
        let mut linker = wasmtime::Linker::new(&self.engine);
        
        // Add WASI imports using the wasmtime 23 preview1 API
        log::debug!("[plugin:wasm_engine] Adding WASI preview1 imports to linker...");
        preview1::add_to_linker_sync(&mut linker, |state: &mut WasmRuntimeState| &mut state.wasi)
            .map_err(|e| {
                log::error!("[plugin:wasm_engine] Failed to add WASI imports: {}", e);
                PluginError::new("link_failed", &format!("Failed to add WASI imports: {}", e))
            })?;
        log::info!("[plugin:wasm_engine] WASI imports added to linker");
        
        // Add host functions to linker
        log::debug!("[plugin:wasm_engine] Adding host functions to linker...");
        self.add_host_functions(&mut linker, &mut store)?;
        log::info!("[plugin:wasm_engine] Host functions added to linker");
        
        // Instantiate the module
        log::debug!("[plugin:wasm_engine] Instantiating WASM module...");
        let instance = linker.instantiate(&mut store, &module)
            .map_err(|e| {
                log::error!("[plugin:wasm_engine] Failed to instantiate module: {}", e);
                PluginError::new("instantiate_failed", &format!("Failed to instantiate module: {}", e))
            })?;
        log::info!("[plugin:wasm_engine] WASM module instantiated successfully");
        
        // Get memory export
        log::debug!("[plugin:wasm_engine] Getting memory export...");
        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| {
                log::error!("[plugin:wasm_engine] Plugin does not export 'memory'");
                PluginError::new("no_memory", "Plugin does not export 'memory'")
            })?;
        log::info!("[plugin:wasm_engine] Memory export obtained: {} pages", memory.size(&store));
        
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
        
        log::info!("[plugin:wasm_engine] Creating WasmPlugin struct for {}", manifest.id);
        let mut plugin = WasmPlugin {
            manifest,
            instance,
            store,
            memory,
            metadata,
        };
        
        // Initialize the plugin
        log::info!("[plugin:wasm_engine] Initializing plugin...");
        plugin.initialize()?;
        log::info!("[plugin:wasm_engine] Plugin {} initialized successfully", plugin.manifest.id);
        
        Ok(plugin)
    }
    
    /// Build WASI context with appropriate permissions
    fn build_wasi_context(&self) -> Result<WasiP1Ctx, Box<dyn std::error::Error>> {
        log::debug!("[plugin:wasm_engine] Building WASI context with permissions: fs={}, network={}, env={}",
            self.config.allow_fs, self.config.allow_network, self.config.allow_env);
        
        let mut builder = WasiCtxBuilder::new();
        
        // Set stdin/stdout/stderr to inherit from parent
        builder.inherit_stdio();
        log::debug!("[plugin:wasm_engine] Inherited stdio");
        
        if self.config.allow_fs {
            log::debug!("[plugin:wasm_engine] Preopening plugin directory: {:?}", self.config.plugin_dir);
            // In wasmtime 23 preview1, we use build_p1() for core wasm modules
            builder.preopened_dir(&self.config.plugin_dir, ".", wasmtime_wasi::DirPerms::all(), wasmtime_wasi::FilePerms::all())?;
            
            // Preopen additional directories
            for dir in &self.config.preopened_dirs {
                let name = dir.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("dir");
                log::debug!("[plugin:wasm_engine] Preopening additional directory: {:?} as {}", dir, name);
                builder.preopened_dir(dir, name, wasmtime_wasi::DirPerms::all(), wasmtime_wasi::FilePerms::all())?;
            }
        }
        
        if self.config.allow_network {
            log::debug!("[plugin:wasm_engine] Inheriting network");
            builder.inherit_network();
        }
        
        if self.config.allow_env {
            log::debug!("[plugin:wasm_engine] Inheriting environment");
            builder.inherit_env();
        }
        
        // Build as WASIp1 context for core wasm modules
        let ctx = builder.build_p1();
        log::info!("[plugin:wasm_engine] WASI context built successfully");
        Ok(ctx)
    }
    
    /// Add host functions that plugins can call
    fn add_host_functions(
        &self,
        linker: &mut wasmtime::Linker<WasmRuntimeState>,
        store: &mut Store<WasmRuntimeState>,
    ) -> Result<(), PluginError> {
        log::info!("[plugin:wasm_engine] Adding host functions to WASM linker");
        
        // Log function: sak_log(ptr: i32, len: i32) -> ()
        log::debug!("[plugin:wasm_engine] Defining host function: sak_log");
        let log_func = Func::wrap(&mut *store,
            |caller: wasmtime::Caller<'_, WasmRuntimeState>, ptr: i32, len: i32| {
                let plugin_id = caller.data().plugin_id.clone();
                log::debug!("[plugin:wasm_engine] Host function sak_log called by plugin {}: ptr={}, len={}", 
                    plugin_id, ptr, len);
                Ok::<(), Error>(())
            }
        );
        linker.define(&mut *store, "env", "sak_log", log_func)
            .map_err(|e| {
                log::error!("[plugin:wasm_engine] Failed to define sak_log: {}", e);
                PluginError::new("link_failed", &format!("Failed to define sak_log: {}", e))
            })?;
        log::debug!("[plugin:wasm_engine] Host function sak_log defined");
        
        // Get editor content: sak_get_content(path_ptr: i32, path_len: i32, out_ptr: i32, out_len: i32) -> i32
        log::debug!("[plugin:wasm_engine] Defining host function: sak_get_content");
        let get_content = Func::wrap(&mut *store,
            |caller: wasmtime::Caller<'_, WasmRuntimeState>,
             path_ptr: i32, path_len: i32, out_ptr: i32, out_len: i32| -> Result<i32, Error> {
                let plugin_id = caller.data().plugin_id.clone();
                log::debug!("[plugin:wasm_engine] Host function sak_get_content called by plugin {}: path_ptr={}, path_len={}, out_ptr={}, out_len={}", 
                    plugin_id, path_ptr, path_len, out_ptr, out_len);
                Ok::<i32, Error>(0i32) // Return bytes written
            }
        );
        linker.define(&mut *store, "env", "sak_get_content", get_content)
            .map_err(|e| {
                log::error!("[plugin:wasm_engine] Failed to define sak_get_content: {}", e);
                PluginError::new("link_failed", &format!("Failed to define sak_get_content: {}", e))
            })?;
        log::debug!("[plugin:wasm_engine] Host function sak_get_content defined");
        
        // Set editor content: sak_set_content(path_ptr: i32, path_len: i32, content_ptr: i32, content_len: i32) -> i32
        log::debug!("[plugin:wasm_engine] Defining host function: sak_set_content");
        let set_content = Func::wrap(&mut *store,
            |caller: wasmtime::Caller<'_, WasmRuntimeState>,
             path_ptr: i32, path_len: i32, content_ptr: i32, content_len: i32| -> Result<i32, Error> {
                let plugin_id = caller.data().plugin_id.clone();
                log::debug!("[plugin:wasm_engine] Host function sak_set_content called by plugin {}: path_ptr={}, path_len={}, content_ptr={}, content_len={}", 
                    plugin_id, path_ptr, path_len, content_ptr, content_len);
                Ok::<i32, Error>(0i32) // Return success
            }
        );
        linker.define(&mut *store, "env", "sak_set_content", set_content)
            .map_err(|e| {
                log::error!("[plugin:wasm_engine] Failed to define sak_set_content: {}", e);
                PluginError::new("link_failed", &format!("Failed to define sak_set_content: {}", e))
            })?;
        log::debug!("[plugin:wasm_engine] Host function sak_set_content defined");
        
        // Show notification: sak_show_notification(title_ptr: i32, title_len: i32, msg_ptr: i32, msg_len: i32) -> ()
        log::debug!("[plugin:wasm_engine] Defining host function: sak_show_notification");
        let show_notification = Func::wrap(&mut *store,
            |caller: wasmtime::Caller<'_, WasmRuntimeState>,
             title_ptr: i32, title_len: i32, msg_ptr: i32, msg_len: i32| -> Result<(), Error> {
                let plugin_id = caller.data().plugin_id.clone();
                log::debug!("[plugin:wasm_engine] Host function sak_show_notification called by plugin {}: title_ptr={}, title_len={}, msg_ptr={}, msg_len={}", 
                    plugin_id, title_ptr, title_len, msg_ptr, msg_len);
                Ok::<(), Error>(())
            }
        );
        linker.define(&mut *store, "env", "sak_show_notification", show_notification)
            .map_err(|e| {
                log::error!("[plugin:wasm_engine] Failed to define sak_show_notification: {}", e);
                PluginError::new("link_failed", &format!("Failed to define sak_show_notification: {}", e))
            })?;
        log::debug!("[plugin:wasm_engine] Host function sak_show_notification defined");
        
        // Get plugin setting: sak_get_setting(key_ptr: i32, key_len: i32, out_ptr: i32, out_len: i32) -> i32
        log::debug!("[plugin:wasm_engine] Defining host function: sak_get_setting");
        let get_setting = Func::wrap(&mut *store,
            |caller: wasmtime::Caller<'_, WasmRuntimeState>,
             key_ptr: i32, key_len: i32, out_ptr: i32, out_len: i32| -> Result<i32, Error> {
                let plugin_id = caller.data().plugin_id.clone();
                log::debug!("[plugin:wasm_engine] Host function sak_get_setting called by plugin {}: key_ptr={}, key_len={}, out_ptr={}, out_len={}", 
                    plugin_id, key_ptr, key_len, out_ptr, out_len);
                Ok::<i32, Error>(0i32)
            }
        );
        linker.define(&mut *store, "env", "sak_get_setting", get_setting)
            .map_err(|e| {
                log::error!("[plugin:wasm_engine] Failed to define sak_get_setting: {}", e);
                PluginError::new("link_failed", &format!("Failed to define sak_get_setting: {}", e))
            })?;
        log::debug!("[plugin:wasm_engine] Host function sak_get_setting defined");
        
        // Set plugin setting: sak_set_setting(key_ptr: i32, key_len: i32, value_ptr: i32, value_len: i32) -> i32
        log::debug!("[plugin:wasm_engine] Defining host function: sak_set_setting");
        let set_setting = Func::wrap(&mut *store,
            |caller: wasmtime::Caller<'_, WasmRuntimeState>,
             key_ptr: i32, key_len: i32, value_ptr: i32, value_len: i32| -> Result<i32, Error> {
                let plugin_id = caller.data().plugin_id.clone();
                log::debug!("[plugin:wasm_engine] Host function sak_set_setting called by plugin {}: key_ptr={}, key_len={}, value_ptr={}, value_len={}", 
                    plugin_id, key_ptr, key_len, value_ptr, value_len);
                Ok::<i32, Error>(0i32)
            }
        );
        linker.define(&mut *store, "env", "sak_set_setting", set_setting)
            .map_err(|e| {
                log::error!("[plugin:wasm_engine] Failed to define sak_set_setting: {}", e);
                PluginError::new("link_failed", &format!("Failed to define sak_set_setting: {}", e))
            })?;
        log::debug!("[plugin:wasm_engine] Host function sak_set_setting defined");
        
        log::info!("[plugin:wasm_engine] All host functions added to linker successfully");
        Ok(())
    }
}

impl WasmPlugin {
    /// Initialize the plugin
    fn initialize(&mut self) -> Result<(), PluginError> {
        log::info!("[plugin:wasm_engine] Initializing WASM plugin: {}", self.manifest.id);
        
        // Look for __initialize or _initialize export
        let init_export_names = ["__initialize", "_initialize", "initialize"];
        for name in &init_export_names {
            if let Some(init) = self.instance.get_func(&mut self.store, name) {
                log::info!("[plugin:wasm_engine] Found init function '{}' for plugin {}", name, self.manifest.id);
                init.typed::<(), ()>(&mut self.store)
                    .map_err(|e| {
                        log::error!("[plugin:wasm_engine] Failed to get init signature for '{}': {}", name, e);
                        PluginError::new("init_failed", &format!("Failed to get init signature: {}", e))
                    })?
                    .call(&mut self.store, ())
                    .map_err(|e| {
                        log::error!("[plugin:wasm_engine] Failed to call initialize '{}': {}", name, e);
                        PluginError::new("init_failed", &format!("Failed to call initialize: {}", e))
                    })?;
                log::info!("[plugin:wasm_engine] Init function '{}' called successfully", name);
                return Ok(());
            }
        }
        
        log::debug!("[plugin:wasm_engine] No init function found for plugin {}", self.manifest.id);
        Ok(())
    }
    
    /// Execute a capability by name
    pub fn execute_capability(&mut self, capability_id: &str, input: &str) -> Result<String, PluginError> {
        log::info!("[plugin:wasm_engine] Executing capability: {}.{}", self.manifest.id, capability_id);
        log::debug!("[plugin:wasm_engine] Capability input: {} bytes", input.len());
        
        // Look for exported function matching capability_id
        let func_name = format!("__capability_{}", capability_id.replace("-", "_"));
        log::debug!("[plugin:wasm_engine] Looking for exported function: {}", func_name);
        
        if let Some(func) = self.instance.get_func(&mut self.store, &func_name) {
            log::info!("[plugin:wasm_engine] Found capability function '{}' for {}", func_name, capability_id);
            
            // Write input to memory
            let input_bytes = input.as_bytes();
            log::debug!("[plugin:wasm_engine] Allocating memory for input: {} bytes", input_bytes.len());
            let input_ptr = self.allocate_memory(input_bytes.len())?;
            log::debug!("[plugin:wasm_engine] Writing input to memory at offset {}", input_ptr);
            self.write_memory(input_ptr, input_bytes)?;
            
            // Call function
            log::debug!("[plugin:wasm_engine] Calling capability function with ptr={}, len={}", input_ptr, input_bytes.len());
            func.typed::<(i32, i32), i32>(&mut self.store)
                .map_err(|e| {
                    log::error!("[plugin:wasm_engine] Failed to get function type: {}", e);
                    PluginError::new("exec_failed", &format!("Failed to get function type: {}", e))
                })?
                .call(&mut self.store, (input_ptr as i32, input_bytes.len() as i32))
                .map_err(|e| {
                    log::error!("[plugin:wasm_engine] Function call failed: {}", e);
                    PluginError::new("exec_failed", &format!("Function call failed: {}", e))
                })?;
            
            // Free allocated memory
            log::debug!("[plugin:wasm_engine] Freeing allocated memory at {}", input_ptr);
            self.free_memory(input_ptr, input_bytes.len())?;
            
            log::info!("[plugin:wasm_engine] Capability {}.{} executed successfully", self.manifest.id, capability_id);
            Ok("{}".to_string()) // Return empty JSON for now
        } else {
            log::error!("[plugin:wasm_engine] Capability '{}' not found (looked for function '{}')", capability_id, func_name);
            Err(PluginError::new("capability_not_found", &format!("Capability '{}' not found", capability_id)))
        }
    }
    
    /// Handle editor event
    pub fn on_event(&mut self, event: &EditorEvent) -> Result<(), PluginError> {
        log::debug!("[plugin:wasm_engine] Handling event {:?} for plugin {}", event, self.manifest.id);
        
        if let Some(handler) = self.instance.get_func(&mut self.store, "__on_event") {
            log::debug!("[plugin:wasm_engine] Found __on_event handler");
            
            // Serialize event to JSON
            let event_json = serde_json::to_string(event)
                .map_err(|e| {
                    log::error!("[plugin:wasm_engine] Failed to serialize event: {}", e);
                    PluginError::new("event_serialize", &e.to_string())
                })?;
            log::debug!("[plugin:wasm_engine] Serialized event: {}", event_json);
            
            // Write to memory and call handler
            let event_bytes = event_json.as_bytes();
            log::debug!("[plugin:wasm_engine] Allocating memory for event: {} bytes", event_bytes.len());
            let event_ptr = self.allocate_memory(event_bytes.len())?;
            self.write_memory(event_ptr, event_bytes)?;
            
            log::debug!("[plugin:wasm_engine] Calling __on_event with ptr={}, len={}", event_ptr, event_bytes.len());
            handler.typed::<(i32, i32), ()>(&mut self.store)
                .map_err(|e| {
                    log::error!("[plugin:wasm_engine] Failed to get event handler type: {}", e);
                    PluginError::new("event_handler", &format!("Failed to get event handler type: {}", e))
                })?
                .call(&mut self.store, (event_ptr as i32, event_bytes.len() as i32))
                .map_err(|e| {
                    log::error!("[plugin:wasm_engine] Event handler failed: {}", e);
                    PluginError::new("event_handler", &format!("Event handler failed: {}", e))
                })?;
            
            self.free_memory(event_ptr, event_bytes.len())?;
            log::debug!("[plugin:wasm_engine] Event handled successfully");
        } else {
            log::debug!("[plugin:wasm_engine] No __on_event handler found for plugin {}", self.manifest.id);
        }
        
        Ok(())
    }
    
    /// Shutdown the plugin
    pub fn shutdown(&mut self) -> Result<(), PluginError> {
        log::info!("[plugin:wasm_engine] Shutting down plugin: {}", self.manifest.id);
        
        if let Some(shutdown) = self.instance.get_func(&mut self.store, "__shutdown") {
            log::debug!("[plugin:wasm_engine] Found __shutdown function, calling it...");
            shutdown.typed::<(), ()>(&mut self.store)
                .map_err(|e| {
                    log::error!("[plugin:wasm_engine] Failed to get shutdown signature: {}", e);
                    PluginError::new("shutdown_failed", &format!("Failed to get shutdown signature: {}", e))
                })?
                .call(&mut self.store, ())
                .map_err(|e| {
                    log::error!("[plugin:wasm_engine] Shutdown failed: {}", e);
                    PluginError::new("shutdown_failed", &format!("Shutdown failed: {}", e))
                })?;
            log::info!("[plugin:wasm_engine] Plugin {} shutdown gracefully", self.manifest.id);
        } else {
            log::debug!("[plugin:wasm_engine] No __shutdown function found for plugin {}", self.manifest.id);
        }
        
        Ok(())
    }
    
    /// Allocate memory in WASM linear memory
    fn allocate_memory(&mut self, size: usize) -> Result<usize, PluginError> {
        log::debug!("[plugin:wasm_engine] Allocating {} bytes in WASM memory", size);
        
        // Try to call exported alloc function
        if let Some(alloc) = self.instance.get_func(&mut self.store, "__alloc") {
            log::debug!("[plugin:wasm_engine] Found __alloc function");
            let result = alloc.typed::<i32, i32>(&mut self.store)
                .map_err(|e| {
                    log::error!("[plugin:wasm_engine] Failed to get alloc signature: {}", e);
                    PluginError::new("alloc_failed", &format!("Failed to get alloc signature: {}", e))
                })?
                .call(&mut self.store, size as i32)
                .map_err(|e| {
                    log::error!("[plugin:wasm_engine] Allocation failed: {}", e);
                    PluginError::new("alloc_failed", &format!("Allocation failed: {}", e))
                })?;
            
            log::debug!("[plugin:wasm_engine] Allocated memory at offset {}", result);
            Ok(result as usize)
        } else {
            log::warn!("[plugin:wasm_engine] No __alloc function found, using fallback (offset 0)");
            // Fallback: just return 0 and let caller handle it
            // In a real implementation, we'd need a proper allocator
            Ok(0)
        }
    }
    
    /// Free allocated memory
    fn free_memory(&mut self, ptr: usize, size: usize) -> Result<(), PluginError> {
        log::debug!("[plugin:wasm_engine] Freeing memory at {} (size {})", ptr, size);
        
        if let Some(dealloc) = self.instance.get_func(&mut self.store, "__dealloc") {
            log::debug!("[plugin:wasm_engine] Found __dealloc function");
            dealloc.typed::<(i32, i32), ()>(&mut self.store)
                .map_err(|e| {
                    log::error!("[plugin:wasm_engine] Failed to get dealloc signature: {}", e);
                    PluginError::new("dealloc_failed", &format!("Failed to get dealloc signature: {}", e))
                })?
                .call(&mut self.store, (ptr as i32, size as i32))
                .map_err(|e| {
                    log::error!("[plugin:wasm_engine] Deallocation failed: {}", e);
                    PluginError::new("dealloc_failed", &format!("Deallocation failed: {}", e))
                })?;
            log::debug!("[plugin:wasm_engine] Memory freed successfully");
        } else {
            log::debug!("[plugin:wasm_engine] No __dealloc function found, skipping free");
        }
        
        Ok(())
    }
    
    /// Write data to WASM memory
    fn write_memory(&mut self, offset: usize, data: &[u8]) -> Result<(), PluginError> {
        log::debug!("[plugin:wasm_engine] Writing {} bytes to WASM memory at offset {}", data.len(), offset);
        
        let mem = self.instance.get_memory(&mut self.store, "memory")
            .ok_or_else(|| {
                log::error!("[plugin:wasm_engine] Memory export not found");
                PluginError::new("no_memory", "Memory export not found")
            })?;
        
        mem.write(&mut self.store, offset, data)
            .map_err(|e| {
                log::error!("[plugin:wasm_engine] Failed to write to memory: {}", e);
                PluginError::new("write_failed", &format!("Failed to write to memory: {}", e))
            })?;
        
        log::debug!("[plugin:wasm_engine] Memory write successful");
        Ok(())
    }
    
    /// Read data from WASM memory
    #[allow(dead_code)]
    fn read_memory(&mut self, offset: usize, len: usize) -> Result<Vec<u8>, PluginError> {
        log::debug!("[plugin:wasm_engine] Reading {} bytes from WASM memory at offset {}", len, offset);
        
        let mem = self.instance.get_memory(&mut self.store, "memory")
            .ok_or_else(|| {
                log::error!("[plugin:wasm_engine] Memory export not found");
                PluginError::new("no_memory", "Memory export not found")
            })?;
        
        let mut buffer = vec![0u8; len];
        mem.read(&mut self.store, offset, &mut buffer)
            .map_err(|e| {
                log::error!("[plugin:wasm_engine] Failed to read from memory: {}", e);
                PluginError::new("read_failed", &format!("Failed to read from memory: {}", e))
            })?;
        
        log::debug!("[plugin:wasm_engine] Memory read successful: {} bytes", buffer.len());
        Ok(buffer)
    }
}
