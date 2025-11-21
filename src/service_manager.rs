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

//! Service management using trait-based providers
//! 
//! This module provides a unified interface for managing services across different platforms
//! and service management systems using a trait-based architecture.

use crate::config::{ServerManagement, ServiceType};
use crate::service_providers::{
    ServiceProvider, ServiceError,
    SystemdProvider, LaunchdProvider, WindowsServiceProvider, 
    PowerShellProvider, CustomScriptProvider
};
use log::{debug, warn};
use std::sync::Arc;

/// Service manager that delegates to trait-based providers
#[derive(Debug, Clone)]
pub struct ServiceManager {
    provider: Arc<dyn ServiceProvider>,
}

impl ServiceManager {
    /// Create a new service manager from configuration
    /// 
    /// This factory method creates the appropriate service provider based on the
    /// service type specified in the configuration.
    pub fn new(config: Option<ServerManagement>) -> Result<Self, ServiceError> {
        let config = config.unwrap_or_default();
        
        let provider: Arc<dyn ServiceProvider> = match &config.service_type {
            ServiceType::Systemd => {
                Arc::new(SystemdProvider::new(&config))
            },
            ServiceType::Launchd => {
                Arc::new(LaunchdProvider::new(&config)?)
            },
            ServiceType::WindowsService => {
                Arc::new(WindowsServiceProvider::new(&config))
            },
            ServiceType::PowerShell => {
                Arc::new(PowerShellProvider::new(&config)?)
            },
            ServiceType::CustomScript => {
                Arc::new(CustomScriptProvider::new(&config)?)
            },
        };

        // Check if the provider is available on this system
        if !provider.is_available() {
            warn!("Service provider '{}' may not be available on this system", provider.provider_name());
        }

        debug!("Using service provider: {}", provider.provider_name());
        
        Ok(Self { provider })
    }

    /// Create a service manager with a custom provider
    /// 
    /// This allows users to inject their own service provider implementations
    /// for testing or custom service management systems.
    pub fn with_provider<P: ServiceProvider + 'static>(provider: P) -> Self {
        Self {
            provider: Arc::new(provider),
        }
    }

    /// Start the service
    pub async fn start_service(&self) -> Result<String, ServiceError> {
        debug!("Starting service using provider: {}", self.provider.provider_name());
        self.provider.start().await
    }

    /// Stop the service gracefully
    pub async fn stop_service(&self) -> Result<String, ServiceError> {
        debug!("Stopping service using provider: {}", self.provider.provider_name());
        self.provider.stop().await
    }

    /// Force stop the service
    /// 
    /// This will attempt to force stop the service, potentially causing data loss.
    /// If force stop is not supported or fails, it will fall back to a graceful stop.
    pub async fn force_stop_service(&self) -> Result<String, ServiceError> {
        debug!("Force stopping service using provider: {}", self.provider.provider_name());
        
        match self.provider.force_stop().await {
            Ok(result) => Ok(result),
            Err(e) => {
                warn!("Force stop failed, attempting graceful stop: {}", e);
                self.provider.stop().await
            }
        }
    }

    /// Get information about the current provider
    pub fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            name: self.provider.provider_name(),
            available: self.provider.is_available(),
        }
    }
}

/// Information about a service provider
#[derive(Debug)]
pub struct ProviderInfo {
    pub name: &'static str,
    pub available: bool,
}
