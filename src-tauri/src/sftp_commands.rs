//! SFTP Site Manager Tauri Commands
//!
//! Exposes site management functionality to the frontend

#![allow(dead_code)]

use serde::Deserialize;
use serde_json::Value;
use crate::sftp_site_manager::{SiteManager, SftpSite};
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref SITE_MANAGER: Mutex<SiteManager> = Mutex::new(SiteManager::new());
}

/// List all saved sites
#[tauri::command]
pub fn sftp_list_sites() -> Result<Value, String> {
    let manager = SITE_MANAGER.lock().map_err(|e| e.to_string())?;
    
    let sites = manager.list_sites();
    let groups = manager.groups.clone();
    
    Ok(serde_json::json!({
        "sites": sites.iter().map(|s| {
            serde_json::json!({
                "id": s.id,
                "name": s.name,
                "hostname": s.hostname,
                "port": s.port,
                "username": s.username,
                "default_path": s.default_path,
                "group": s.group,
                "last_connected": s.last_connected,
                "notes": s.notes,
            })
        }).collect::<Vec<_>>(),
        "groups": groups,
    }))
}

/// Add new site
#[derive(Deserialize)]
pub struct AddSiteRequest {
    site: SftpSite,
    password: Option<String>,
    ssh_key_path: Option<String>,
}

#[tauri::command]
pub fn sftp_add_site(request: AddSiteRequest) -> Result<Value, String> {
    let mut manager = SITE_MANAGER.lock().map_err(|e| e.to_string())?;
    
    let mut site = request.site;
    
    // Set password if provided
    if let Some(password) = request.password {
        site.set_password(password);
    }
    
    // Set SSH key if provided
    if let Some(key_path) = request.ssh_key_path {
        site.ssh_key_path = Some(key_path);
    }
    
    manager.add_site(site).map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "success": true
    }))
}

/// Update existing site
#[tauri::command]
pub fn sftp_update_site(site: SftpSite, password: Option<String>) -> Result<Value, String> {
    let mut manager = SITE_MANAGER.lock().map_err(|e| e.to_string())?;
    
    // If password provided, update it
    if let Some(pass) = password {
        let mut updated_site = site.clone();
        updated_site.set_password(pass);
        manager.update_site(updated_site).map_err(|e| e.to_string())?;
    } else {
        manager.update_site(site).map_err(|e| e.to_string())?;
    }
    
    Ok(serde_json::json!({
        "success": true
    }))
}

/// Remove site
#[tauri::command]
pub fn sftp_remove_site(site_id: String) -> Result<Value, String> {
    let mut manager = SITE_MANAGER.lock().map_err(|e| e.to_string())?;
    manager.remove_site(&site_id).map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "success": true
    }))
}

/// Connect to site
#[tauri::command]
pub async fn sftp_connect_site(site_id: String) -> Result<Value, String> {
    let manager = SITE_MANAGER.lock().map_err(|e| e.to_string())?;
    
    let site = manager.get_site(&site_id)
        .ok_or_else(|| "Site not found".to_string())?;
    
    // Try to connect using modular system
    let result = crate::modular::execute_module(
        "sftp",
        "connect",
        serde_json::json!({
            "connection_id": site_id,
            "hostname": site.hostname,
            "port": site.port,
            "username": site.username,
            "password": site.get_password(),
        })
    );
    
    match result {
        Ok(_) => {
            // Update last connected
            drop(manager);
            let mut manager = SITE_MANAGER.lock().map_err(|e| e.to_string())?;
            manager.mark_connected(&site_id).map_err(|e| e.to_string())?;
            
            Ok(serde_json::json!({
                "success": true,
                "message": "Connected successfully"
            }))
        }
        Err(e) => Err(format!("Failed to connect: {}", e)),
    }
}

/// Test connection without saving
#[tauri::command]
pub async fn sftp_test_connection(site: SftpSite) -> Result<Value, String> {
    let manager = SiteManager::new();
    
    match manager.test_connection(&site).await {
        Ok(true) => Ok(serde_json::json!({
            "success": true,
            "message": "Connection successful"
        })),
        Ok(false) => Ok(serde_json::json!({
            "success": false,
            "message": "Connection failed"
        })),
        Err(e) => Err(e.to_string()),
    }
}
