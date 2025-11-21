// 
// PalConnect - A Discord bot for PalWorld server monitoring
// Copyright (C) 2025  Lily Ana Valley <hi@lilyvalley.dev> <https://lilyvalley.dev>
//
// This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General 
// Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) 
// any later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied 
// warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more
// details.
// 
// You should have received a copy of the GNU Affero General Public License along with this program.  If not, see
// <https://www.gnu.org/licenses/>.
// 

//! Service provider implementations for different platforms
//! 
//! This module contains trait definitions and implementations for managing
//! services across different platforms and service management systems.

use async_trait::async_trait;
use log::{debug, warn, error};
use std::process::{Command, Output};
use crate::config::ServerManagement;

/// Error types for service operations
#[derive(Debug)]
pub enum ServiceError {
    CommandFailed(String),
    UnsupportedOperation(String),
    ConfigurationError(String),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            ServiceError::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {}", msg),
            ServiceError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}

/// Trait for service management providers
/// 
/// Implement this trait to add support for new service management systems.
/// All methods are async to support both synchronous and asynchronous operations.
#[async_trait]
pub trait ServiceProvider: Send + Sync + std::fmt::Debug {
    /// Start the service
    async fn start(&self) -> Result<String, ServiceError>;
    
    /// Stop the service gracefully
    async fn stop(&self) -> Result<String, ServiceError>;
    
    /// Force stop the service (may cause data loss)
    /// Default implementation tries graceful stop first
    async fn force_stop(&self) -> Result<String, ServiceError> {
        self.stop().await
    }
    
    /// Get the display name for this service provider
    fn provider_name(&self) -> &'static str;
    
    /// Check if this provider is available on the current system
    /// Default implementation returns true
    fn is_available(&self) -> bool {
        true
    }
}

/// Helper trait for command execution
trait CommandExecutor {
    fn execute_command(&self, output: Output, operation: &str) -> Result<String, ServiceError> {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let result = format!("{} completed successfully. Output: {}", operation, stdout.trim());
            debug!("{}", result);
            Ok(result)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let error_msg = format!(
                "{} failed with exit code: {:?}. Stderr: {} Stdout: {}", 
                operation, 
                output.status.code(), 
                stderr.trim(),
                stdout.trim()
            );
            error!("{}", error_msg);
            Err(ServiceError::CommandFailed(error_msg))
        }
    }
    
    fn parse_and_execute(&self, command: &str) -> Result<Output, ServiceError> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ServiceError::ConfigurationError("Empty command".to_string()));
        }

        let mut cmd = Command::new(parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }

        cmd.output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute command '{}': {}", command, e)))
    }
}

/// Linux systemd service provider
#[derive(Debug, Clone)]
pub struct SystemdProvider {
    service_name: String,
}

impl SystemdProvider {
    pub fn new(config: &ServerManagement) -> Self {
        let service_name = config.service_name
            .as_ref()
            .map(|s| s.clone())
            .unwrap_or_else(|| "palworld.service".to_string());
        
        Self { service_name }
    }
}

impl CommandExecutor for SystemdProvider {}

#[async_trait]
impl ServiceProvider for SystemdProvider {
    async fn start(&self) -> Result<String, ServiceError> {
        debug!("Starting systemd service: {}", self.service_name);
        let output = Command::new("systemctl")
            .arg("start")
            .arg(&self.service_name)
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute systemctl: {}", e)))?;
        
        self.execute_command(output, "systemd start")
    }

    async fn stop(&self) -> Result<String, ServiceError> {
        debug!("Stopping systemd service: {}", self.service_name);
        let output = Command::new("systemctl")
            .arg("stop")
            .arg(&self.service_name)
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute systemctl: {}", e)))?;
        
        self.execute_command(output, "systemd stop")
    }

    async fn force_stop(&self) -> Result<String, ServiceError> {
        debug!("Force killing systemd service: {}", self.service_name);
        let output = Command::new("systemctl")
            .arg("kill")
            .arg("--signal=SIGKILL")
            .arg(&self.service_name)
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute systemctl kill: {}", e)))?;
        
        self.execute_command(output, "systemd force stop")
    }

    fn provider_name(&self) -> &'static str {
        "systemd"
    }

    fn is_available(&self) -> bool {
        Command::new("systemctl").arg("--version").output().is_ok()
    }
}

/// macOS launchd service provider
#[derive(Debug, Clone)]
pub struct LaunchdProvider {
    service_path: String,
}

impl LaunchdProvider {
    pub fn new(config: &ServerManagement) -> Result<Self, ServiceError> {
        let service_path = config.service_name
            .as_ref()
            .ok_or_else(|| ServiceError::ConfigurationError(
                "service_name (plist path) is required for launchd".to_string()
            ))?
            .clone();
        
        Ok(Self { service_path })
    }
}

impl CommandExecutor for LaunchdProvider {}

#[async_trait]
impl ServiceProvider for LaunchdProvider {
    async fn start(&self) -> Result<String, ServiceError> {
        debug!("Starting launchd service: {}", self.service_path);
        let output = Command::new("launchctl")
            .arg("load")
            .arg("-w")
            .arg(&self.service_path)
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute launchctl: {}", e)))?;
        
        self.execute_command(output, "launchd start")
    }

    async fn stop(&self) -> Result<String, ServiceError> {
        debug!("Stopping launchd service: {}", self.service_path);
        let output = Command::new("launchctl")
            .arg("unload")
            .arg("-w")
            .arg(&self.service_path)
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute launchctl: {}", e)))?;
        
        self.execute_command(output, "launchd stop")
    }

    async fn force_stop(&self) -> Result<String, ServiceError> {
        debug!("Force stopping launchd service: {}", self.service_path);
        
        // First try to stop gracefully
        let _ = self.stop().await;
        
        // For launchd, graceful stop is usually sufficient
        Ok("Force stopped launchd service (attempted graceful stop)".to_string())
    }

    fn provider_name(&self) -> &'static str {
        "launchd"
    }

    fn is_available(&self) -> bool {
        Command::new("launchctl").arg("version").output().is_ok()
    }
}

/// Windows Service provider
#[derive(Debug, Clone)]
pub struct WindowsServiceProvider {
    service_name: String,
}

impl WindowsServiceProvider {
    pub fn new(config: &ServerManagement) -> Self {
        let service_name = config.service_name
            .as_ref()
            .map(|s| s.clone())
            .unwrap_or_else(|| "PalWorldServer".to_string());
        
        Self { service_name }
    }
}

impl CommandExecutor for WindowsServiceProvider {}

#[async_trait]
impl ServiceProvider for WindowsServiceProvider {
    async fn start(&self) -> Result<String, ServiceError> {
        debug!("Starting Windows service: {}", self.service_name);
        let output = Command::new("sc")
            .arg("start")
            .arg(&self.service_name)
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute sc: {}", e)))?;
        
        self.execute_command(output, "Windows service start")
    }

    async fn stop(&self) -> Result<String, ServiceError> {
        debug!("Stopping Windows service: {}", self.service_name);
        let output = Command::new("sc")
            .arg("stop")
            .arg(&self.service_name)
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute sc: {}", e)))?;
        
        self.execute_command(output, "Windows service stop")
    }

    async fn force_stop(&self) -> Result<String, ServiceError> {
        debug!("Force stopping Windows service: {}", self.service_name);
        
        // Try to stop the service forcefully using taskkill if sc stop doesn't work
        let sc_output = Command::new("sc")
            .arg("stop")
            .arg(&self.service_name)
            .output();
        
        if sc_output.is_err() || !sc_output.as_ref().unwrap().status.success() {
            warn!("sc stop failed, trying taskkill");
            // Alternative: try to kill the process by name
            let output = Command::new("taskkill")
                .arg("/F")
                .arg("/IM")
                .arg("PalServer.exe") // Adjust this to the actual executable name
                .output()
                .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute taskkill: {}", e)))?;
            
            self.execute_command(output, "Windows service force stop")
        } else {
            self.execute_command(sc_output.unwrap(), "Windows service stop")
        }
    }

    fn provider_name(&self) -> &'static str {
        "windows_service"
    }

    fn is_available(&self) -> bool {
        Command::new("sc").arg("query").output().is_ok()
    }
}

/// PowerShell script provider
#[derive(Debug, Clone)]
pub struct PowerShellProvider {
    start_command: String,
    stop_command: String,
    force_stop_command: Option<String>,
}

impl PowerShellProvider {
    pub fn new(config: &ServerManagement) -> Result<Self, ServiceError> {
        let start_command = config.start_command
            .as_ref()
            .ok_or_else(|| ServiceError::ConfigurationError(
                "start_command is required for PowerShell service type".to_string()
            ))?
            .clone();
        
        let stop_command = config.stop_command
            .as_ref()
            .ok_or_else(|| ServiceError::ConfigurationError(
                "stop_command is required for PowerShell service type".to_string()
            ))?
            .clone();
        
        let force_stop_command = config.force_stop_command.clone();
        
        Ok(Self {
            start_command,
            stop_command,
            force_stop_command,
        })
    }
}

impl CommandExecutor for PowerShellProvider {}

#[async_trait]
impl ServiceProvider for PowerShellProvider {
    async fn start(&self) -> Result<String, ServiceError> {
        debug!("Starting via PowerShell: {}", self.start_command);
        let output = Command::new("powershell")
            .arg("-Command")
            .arg(&self.start_command)
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute PowerShell: {}", e)))?;
        
        self.execute_command(output, "PowerShell start")
    }

    async fn stop(&self) -> Result<String, ServiceError> {
        debug!("Stopping via PowerShell: {}", self.stop_command);
        let output = Command::new("powershell")
            .arg("-Command")
            .arg(&self.stop_command)
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute PowerShell: {}", e)))?;
        
        self.execute_command(output, "PowerShell stop")
    }

    async fn force_stop(&self) -> Result<String, ServiceError> {
        if let Some(force_command) = &self.force_stop_command {
            debug!("Force stopping via PowerShell: {}", force_command);
            let output = Command::new("powershell")
                .arg("-Command")
                .arg(force_command)
                .output()
                .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute PowerShell: {}", e)))?;
            
            self.execute_command(output, "PowerShell force stop")
        } else {
            // Fallback to regular stop
            self.stop().await
        }
    }

    fn provider_name(&self) -> &'static str {
        "powershell"
    }

    fn is_available(&self) -> bool {
        Command::new("powershell").arg("-Command").arg("$PSVersionTable").output().is_ok()
    }
}

/// Custom script provider
#[derive(Debug, Clone)]
pub struct CustomScriptProvider {
    start_command: String,
    stop_command: String,
    force_stop_command: Option<String>,
}

impl CustomScriptProvider {
    pub fn new(config: &ServerManagement) -> Result<Self, ServiceError> {
        let start_command = config.start_command
            .as_ref()
            .ok_or_else(|| ServiceError::ConfigurationError(
                "start_command is required for custom script service type".to_string()
            ))?
            .clone();
        
        let stop_command = config.stop_command
            .as_ref()
            .ok_or_else(|| ServiceError::ConfigurationError(
                "stop_command is required for custom script service type".to_string()
            ))?
            .clone();
        
        let force_stop_command = config.force_stop_command.clone();
        
        Ok(Self {
            start_command,
            stop_command,
            force_stop_command,
        })
    }
}

impl CommandExecutor for CustomScriptProvider {}

#[async_trait]
impl ServiceProvider for CustomScriptProvider {
    async fn start(&self) -> Result<String, ServiceError> {
        debug!("Starting via custom script: {}", self.start_command);
        let output = self.parse_and_execute(&self.start_command)?;
        self.execute_command(output, "custom script start")
    }

    async fn stop(&self) -> Result<String, ServiceError> {
        debug!("Stopping via custom script: {}", self.stop_command);
        let output = self.parse_and_execute(&self.stop_command)?;
        self.execute_command(output, "custom script stop")
    }

    async fn force_stop(&self) -> Result<String, ServiceError> {
        if let Some(force_command) = &self.force_stop_command {
            debug!("Force stopping via custom script: {}", force_command);
            let output = self.parse_and_execute(force_command)?;
            self.execute_command(output, "custom script force stop")
        } else {
            // Fallback to regular stop
            self.stop().await
        }
    }

    fn provider_name(&self) -> &'static str {
        "custom_script"
    }

    fn is_available(&self) -> bool {
        // Custom scripts are always considered available
        true
    }
}
