//! SFTP Client Implementation
//!
//! Provides SFTP file operations over SSH connections.
//! Uses command execution as a fallback when native SFTP is not available.

use crate::error::SshError;
use crate::ssh::SshClient;
use serde::{Deserialize, Serialize};

/// File entry information from remote server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteFileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub permissions: String,
    pub modified: String,
    pub owner: String,
}

impl SshClient {
    /// List directory contents using ls command
    pub async fn list_directory(&self, path: &str) -> Result<Vec<RemoteFileEntry>, SshError> {
        // Use ls -la with specific format for parsing
        let cmd = format!(
            "ls -la --time-style=long-iso {} 2>/dev/null || ls -la {}",
            shell_escape(path),
            shell_escape(path)
        );
        
        let result = self.execute(&cmd).await?;
        
        if result.exit_code != 0 {
            return Err(SshError::CommandExecution(format!(
                "Failed to list directory: {}",
                result.stderr_string()
            )));
        }
        
        parse_ls_output(&result.stdout_string(), path)
    }

    /// Read file contents
    pub async fn read_file(&self, path: &str) -> Result<Vec<u8>, SshError> {
        let cmd = format!("cat {}", shell_escape(path));
        let result = self.execute(&cmd).await?;
        
        if result.exit_code != 0 {
            return Err(SshError::CommandExecution(format!(
                "Failed to read file: {}",
                result.stderr_string()
            )));
        }
        
        Ok(result.stdout)
    }

    /// Write file contents (base64 encoded for binary safety)
    pub async fn write_file(&self, path: &str, data: &[u8]) -> Result<(), SshError> {
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data);
        let cmd = format!(
            "echo '{}' | base64 -d > {}",
            encoded,
            shell_escape(path)
        );
        
        let result = self.execute(&cmd).await?;
        
        if result.exit_code != 0 {
            return Err(SshError::CommandExecution(format!(
                "Failed to write file: {}",
                result.stderr_string()
            )));
        }
        
        Ok(())
    }

    /// Delete file or directory
    pub async fn delete_path(&self, path: &str, recursive: bool) -> Result<(), SshError> {
        let cmd = if recursive {
            format!("rm -rf {}", shell_escape(path))
        } else {
            format!("rm -f {}", shell_escape(path))
        };
        
        let result = self.execute(&cmd).await?;
        
        if result.exit_code != 0 {
            return Err(SshError::CommandExecution(format!(
                "Failed to delete: {}",
                result.stderr_string()
            )));
        }
        
        Ok(())
    }

    /// Rename/move file or directory
    pub async fn rename_path(&self, old_path: &str, new_path: &str) -> Result<(), SshError> {
        let cmd = format!("mv {} {}", shell_escape(old_path), shell_escape(new_path));
        let result = self.execute(&cmd).await?;
        
        if result.exit_code != 0 {
            return Err(SshError::CommandExecution(format!(
                "Failed to rename: {}",
                result.stderr_string()
            )));
        }
        
        Ok(())
    }

    /// Create directory
    pub async fn create_directory(&self, path: &str) -> Result<(), SshError> {
        let cmd = format!("mkdir -p {}", shell_escape(path));
        let result = self.execute(&cmd).await?;
        
        if result.exit_code != 0 {
            return Err(SshError::CommandExecution(format!(
                "Failed to create directory: {}",
                result.stderr_string()
            )));
        }
        
        Ok(())
    }

    /// Get file/directory info
    pub async fn stat_path(&self, path: &str) -> Result<RemoteFileEntry, SshError> {
        let cmd = format!(
            "stat --format='%n|%F|%s|%a|%Y|%U' {} 2>/dev/null || stat -f '%N|%HT|%z|%Lp|%m|%Su' {}",
            shell_escape(path),
            shell_escape(path)
        );
        
        let result = self.execute(&cmd).await?;
        
        if result.exit_code != 0 {
            return Err(SshError::CommandExecution(format!(
                "Failed to stat path: {}",
                result.stderr_string()
            )));
        }
        
        parse_stat_output(&result.stdout_string(), path)
    }

    /// Check if path exists
    pub async fn path_exists(&self, path: &str) -> Result<bool, SshError> {
        let cmd = format!("test -e {} && echo 'exists'", shell_escape(path));
        let result = self.execute(&cmd).await?;
        Ok(result.stdout_string().trim() == "exists")
    }

    /// Get file size
    pub async fn file_size(&self, path: &str) -> Result<u64, SshError> {
        let cmd = format!(
            "stat --format='%s' {} 2>/dev/null || stat -f '%z' {}",
            shell_escape(path),
            shell_escape(path)
        );
        
        let result = self.execute(&cmd).await?;
        
        if result.exit_code != 0 {
            return Err(SshError::CommandExecution(format!(
                "Failed to get file size: {}",
                result.stderr_string()
            )));
        }
        
        result.stdout_string().trim().parse().map_err(|e| {
            SshError::CommandExecution(format!("Failed to parse file size: {}", e))
        })
    }
}

/// Escape shell special characters
fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Parse ls -la output into file entries
fn parse_ls_output(output: &str, base_path: &str) -> Result<Vec<RemoteFileEntry>, SshError> {
    let mut entries = Vec::new();
    let base_path = base_path.trim_end_matches('/');
    
    for line in output.lines().skip(1) { // Skip "total" line
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // Parse ls -la format: permissions links owner group size date time name
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 8 {
            continue;
        }
        
        let permissions = parts[0];
        let owner = parts[2];
        let size: u64 = parts[4].parse().unwrap_or(0);
        
        // Name is everything after the date/time fields
        let name_start = if parts.len() >= 9 { 8 } else { 7 };
        let name = parts[name_start..].join(" ");
        
        // Skip . and handle ..
        if name == "." {
            continue;
        }
        
        let is_dir = permissions.starts_with('d');
        let path = if name == ".." {
            get_parent_path(base_path)
        } else {
            format!("{}/{}", base_path, name)
        };
        
        // Parse date (simplified)
        let modified = if parts.len() >= 8 {
            format!("{} {}", parts[5], parts[6])
        } else {
            String::new()
        };
        
        entries.push(RemoteFileEntry {
            name,
            path,
            is_dir,
            size,
            permissions: permissions.to_string(),
            modified,
            owner: owner.to_string(),
        });
    }
    
    Ok(entries)
}

/// Parse stat output
fn parse_stat_output(output: &str, path: &str) -> Result<RemoteFileEntry, SshError> {
    let parts: Vec<&str> = output.trim().split('|').collect();
    
    if parts.len() < 6 {
        return Err(SshError::CommandExecution("Invalid stat output".to_string()));
    }
    
    let name = std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());
    
    let is_dir = parts[1].contains("directory") || parts[1].contains("Directory");
    let size: u64 = parts[2].parse().unwrap_or(0);
    let permissions = format!("{:o}", parts[3].parse::<u32>().unwrap_or(0));
    
    Ok(RemoteFileEntry {
        name,
        path: path.to_string(),
        is_dir,
        size,
        permissions,
        modified: parts[4].to_string(),
        owner: parts[5].to_string(),
    })
}

/// Get parent path
fn get_parent_path(path: &str) -> String {
    let path = path.trim_end_matches('/');
    if path.is_empty() || path == "/" {
        return "/".to_string();
    }
    
    match path.rfind('/') {
        Some(0) => "/".to_string(),
        Some(idx) => path[..idx].to_string(),
        None => "/".to_string(),
    }
}
