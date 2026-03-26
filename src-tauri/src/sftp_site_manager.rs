//! SFTP Site Manager
//!
//! Manages saved SFTP connections/sites with encrypted credentials
//! Users can save multiple sites and quickly connect

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use crate::modular::ModuleError;

/// A saved SFTP site configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftpSite {
    pub id: String,
    pub name: String,
    pub hostname: String,
    pub port: u16,
    pub username: String,
    /// Encrypted password - use keyring or simple encryption
    pub password_encrypted: Option<String>,
    /// Path to SSH private key (optional)
    pub ssh_key_path: Option<String>,
    /// Default remote directory
    pub default_path: Option<String>,
    /// Site category/group
    pub group: Option<String>,
    /// Last connected timestamp
    pub last_connected: Option<String>,
    /// Notes about this site
    pub notes: Option<String>,
}

impl SftpSite {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            hostname: String::new(),
            port: 22,
            username: String::new(),
            password_encrypted: None,
            ssh_key_path: None,
            default_path: Some("/home".to_string()),
            group: Some("Default".to_string()),
            last_connected: None,
            notes: None,
        }
    }

    pub fn with_hostname(mut self, hostname: impl Into<String>) -> Self {
        self.hostname = hostname.into();
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_credentials(mut self, username: impl Into<String>, password: Option<impl Into<String>>) -> Self {
        self.username = username.into();
        if let Some(pass) = password {
            self.password_encrypted = Some(encrypt_password(&pass.into()));
        }
        self
    }

    pub fn with_ssh_key(mut self, key_path: impl Into<String>) -> Self {
        self.ssh_key_path = Some(key_path.into());
        self
    }

    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Get decrypted password
    pub fn get_password(&self) -> Option<String> {
        self.password_encrypted.as_ref().map(|enc| decrypt_password(enc))
    }

    /// Set password (auto-encrypts)
    pub fn set_password(&mut self, password: impl Into<String>) {
        self.password_encrypted = Some(encrypt_password(&password.into()));
    }
}

/// Simple XOR encryption for passwords (in production, use proper encryption)
fn encrypt_password(password: &str) -> String {
    let key = b"sak_editor_key_2024";
    let encrypted: Vec<u8> = password
        .bytes()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect();
    base64::encode(&encrypted)
}

fn decrypt_password(encrypted: &str) -> String {
    let key = b"sak_editor_key_2024";
    let bytes = base64::decode(encrypted).unwrap_or_default();
    let decrypted: Vec<u8> = bytes
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect();
    String::from_utf8_lossy(&decrypted).to_string()
}

/// Site Manager - manages all saved SFTP sites
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SiteManager {
    pub sites: HashMap<String, SftpSite>,
    pub groups: Vec<String>,
    version: String,
}

impl SiteManager {
    pub fn new() -> Self {
        Self {
            sites: HashMap::new(),
            groups: vec!["Default".to_string()],
            version: "1.0".to_string(),
        }
    }

    /// Get storage path
    fn storage_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::env::temp_dir());
        config_dir.join("sak-editor").join("sftp_sites.json")
    }

    /// Load sites from disk
    pub fn load() -> Result<Self, ModuleError> {
        let path = Self::storage_path();
        
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| ModuleError::ExecutionFailed(format!("Failed to read sites: {}", e)))?;
        
        let manager: SiteManager = serde_json::from_str(&content)
            .map_err(|e| ModuleError::ExecutionFailed(format!("Failed to parse sites: {}", e)))?;
        
        Ok(manager)
    }

    /// Save sites to disk
    pub fn save(&self) -> Result<(), ModuleError> {
        let path = Self::storage_path();
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ModuleError::ExecutionFailed(format!("Failed to create directory: {}", e)))?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| ModuleError::ExecutionFailed(format!("Failed to serialize: {}", e)))?;
        
        fs::write(&path, json)
            .map_err(|e| ModuleError::ExecutionFailed(format!("Failed to write sites: {}", e)))?;
        
        Ok(())
    }

    /// Add a new site
    pub fn add_site(&mut self, site: SftpSite) -> Result<(), ModuleError> {
        if self.sites.contains_key(&site.id) {
            return Err(ModuleError::ExecutionFailed(format!("Site '{}' already exists", site.id)));
        }
        
        // Add group if new
        if let Some(ref group) = site.group {
            if !self.groups.contains(group) {
                self.groups.push(group.clone());
            }
        }
        
        self.sites.insert(site.id.clone(), site);
        self.save()?;
        Ok(())
    }

    /// Update existing site
    pub fn update_site(&mut self, site: SftpSite) -> Result<(), ModuleError> {
        if !self.sites.contains_key(&site.id) {
            return Err(ModuleError::ExecutionFailed(format!("Site '{}' not found", site.id)));
        }
        
        self.sites.insert(site.id.clone(), site);
        self.save()?;
        Ok(())
    }

    /// Remove a site
    pub fn remove_site(&mut self, site_id: &str) -> Result<(), ModuleError> {
        if self.sites.remove(site_id).is_none() {
            return Err(ModuleError::ExecutionFailed(format!("Site '{}' not found", site_id)));
        }
        self.save()?;
        Ok(())
    }

    /// Get a site by ID
    pub fn get_site(&self, site_id: &str) -> Option<&SftpSite> {
        self.sites.get(site_id)
    }

    /// Get mutable site
    pub fn get_site_mut(&mut self, site_id: &str) -> Option<&mut SftpSite> {
        self.sites.get_mut(site_id)
    }

    /// List all sites
    pub fn list_sites(&self) -> Vec<&SftpSite> {
        self.sites.values().collect()
    }

    /// List sites by group
    pub fn list_sites_by_group(&self, group: &str) -> Vec<&SftpSite> {
        self.sites
            .values()
            .filter(|s| s.group.as_ref() == Some(&group.to_string()))
            .collect()
    }

    /// Update last connected time
    pub fn mark_connected(&mut self, site_id: &str) -> Result<(), ModuleError> {
        if let Some(site) = self.sites.get_mut(site_id) {
            site.last_connected = Some(chrono::Local::now().to_rfc3339());
            self.save()?;
        }
        Ok(())
    }

    /// Add a new group
    pub fn add_group(&mut self, group: impl Into<String>) {
        let group_str = group.into();
        if !self.groups.contains(&group_str) {
            self.groups.push(group_str);
        }
    }

    /// Remove a group
    pub fn remove_group(&mut self, group: &str) {
        self.groups.retain(|g| g != group);
        // Update sites that were in this group
        for site in self.sites.values_mut() {
            if site.group.as_ref() == Some(&group.to_string()) {
                site.group = Some("Default".to_string());
            }
        }
    }

    /// Rename a group
    pub fn rename_group(&mut self, old_name: &str, new_name: impl Into<String>) {
        let new_name = new_name.into();
        if let Some(pos) = self.groups.iter().position(|g| g == old_name) {
            self.groups[pos] = new_name.clone();
        }
        for site in self.sites.values_mut() {
            if site.group.as_ref() == Some(&old_name.to_string()) {
                site.group = Some(new_name.clone());
            }
        }
    }

    /// Import sites from file
    pub fn import_from_file(&mut self, path: &str) -> Result<usize, ModuleError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ModuleError::ExecutionFailed(format!("Failed to read import file: {}", e)))?;
        
        let imported: SiteManager = serde_json::from_str(&content)
            .map_err(|e| ModuleError::ExecutionFailed(format!("Failed to parse import: {}", e)))?;
        
        let count = imported.sites.len();
        for (id, site) in imported.sites {
            self.sites.insert(id, site);
        }
        
        self.save()?;
        Ok(count)
    }

    /// Export sites to file
    pub fn export_to_file(&self, path: &str) -> Result<(), ModuleError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| ModuleError::ExecutionFailed(format!("Failed to serialize: {}", e)))?;
        
        fs::write(path, json)
            .map_err(|e| ModuleError::ExecutionFailed(format!("Failed to write export: {}", e)))?;
        
        Ok(())
    }

    /// Duplicate a site
    pub fn duplicate_site(&mut self, site_id: &str, new_name: impl Into<String>) -> Result<String, ModuleError> {
        let site = self.sites.get(site_id)
            .ok_or_else(|| ModuleError::ExecutionFailed(format!("Site '{}' not found", site_id)))?;
        
        let mut new_site = site.clone();
        new_site.id = uuid::Uuid::new_v4().to_string();
        new_site.name = new_name.into();
        new_site.last_connected = None;
        
        let new_id = new_site.id.clone();
        self.add_site(new_site)?;
        Ok(new_id)
    }

    /// Test connection (without saving)
    pub async fn test_connection(&self, site: &SftpSite) -> Result<bool, ModuleError> {
        // Try to connect using sftp_module
        let result = crate::modular::execute_module(
            "sftp",
            "connect",
            serde_json::json!({
                "hostname": site.hostname,
                "port": site.port,
                "username": site.username,
                "password": site.get_password(),
            })
        );
        
        match result {
            Ok(_) => Ok(true),
            Err(e) => Err(ModuleError::ExecutionFailed(format!("Connection failed: {}", e))),
        }
    }
}

/// Site connection history
#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionHistory {
    pub site_id: String,
    pub connected_at: String,
    pub success: bool,
    pub error_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_encryption() {
        let password = "secret123";
        let encrypted = encrypt_password(password);
        let decrypted = decrypt_password(&encrypted);
        
        assert_eq!(password, decrypted);
        assert_ne!(password, encrypted);
    }

    #[test]
    fn test_site_builder() {
        let site = SftpSite::new("1", "Production Server")
            .with_hostname("prod.example.com")
            .with_port(2222)
            .with_credentials("admin", Some("password123"))
            .with_group("Production");
        
        assert_eq!(site.name, "Production Server");
        assert_eq!(site.hostname, "prod.example.com");
        assert_eq!(site.port, 2222);
        assert_eq!(site.username, "admin");
        assert!(site.password_encrypted.is_some());
        assert_eq!(site.group, Some("Production".to_string()));
    }

    #[test]
    fn test_site_manager() {
        let mut manager = SiteManager::new();
        
        let site = SftpSite::new("1", "Test Server")
            .with_hostname("test.com")
            .with_credentials("user", Some("pass"));
        
        manager.add_site(site).unwrap();
        
        assert_eq!(manager.list_sites().len(), 1);
        assert!(manager.get_site("1").is_some());
        
        manager.remove_site("1").unwrap();
        assert_eq!(manager.list_sites().len(), 0);
    }
}
