# Trait-Based Service Management

PalConnect uses a trait-based architecture for service management, allowing for extensible and testable service providers. This document explains the benefits and how to implement custom service providers.

## Architecture Overview

The service management system is built around the `ServiceProvider` trait, which defines the interface that all service providers must implement:

```rust
#[async_trait]
pub trait ServiceProvider: Send + Sync + std::fmt::Debug {
    async fn start(&self) -> Result<String, ServiceError>;
    async fn stop(&self) -> Result<String, ServiceError>;
    async fn force_stop(&self) -> Result<String, ServiceError>;
    fn provider_name(&self) -> &'static str;
    fn is_available(&self) -> bool;
}
```

## Benefits of the Trait-Based Approach

### 1. **Extensibility**
Users and third-party developers can implement custom service providers without modifying PalConnect's core code:

```rust
use async_trait::async_trait;
use palconnect::service_providers::{ServiceProvider, ServiceError};

#[derive(Debug, Clone)]
pub struct MyCustomProvider {
    config: MyConfig,
}

#[async_trait]
impl ServiceProvider for MyCustomProvider {
    async fn start(&self) -> Result<String, ServiceError> {
        // Custom implementation
    }
    // ... other methods
}
```

### 2. **Testability**
Easy to create mock implementations for unit testing:

```rust
#[derive(Debug)]
pub struct MockServiceProvider {
    should_fail: bool,
}

#[async_trait]
impl ServiceProvider for MockServiceProvider {
    async fn start(&self) -> Result<String, ServiceError> {
        if self.should_fail {
            Err(ServiceError::CommandFailed("Mock failure".to_string()))
        } else {
            Ok("Mock success".to_string())
        }
    }
    // ... other methods
}

// Use in tests
let manager = ServiceManager::with_provider(MockServiceProvider { should_fail: false });
```

### 3. **Separation of Concerns**
Each service type is implemented in its own module with clear responsibilities:

```
service_providers.rs
├── ServiceProvider trait
├── SystemdProvider
├── LaunchdProvider
├── WindowsServiceProvider
├── PowerShellProvider
└── CustomScriptProvider
```

### 4. **Type Safety**
Compile-time guarantees about service capabilities and error handling.

### 5. **Plugin Architecture**
Third-party crates can provide additional service implementations:

```rust
// In external crate: palconnect-k8s-provider
pub struct KubernetesProvider { ... }

impl ServiceProvider for KubernetesProvider { ... }

// In user's application
use palconnect::ServiceManager;
use palconnect_k8s_provider::KubernetesProvider;

let k8s_provider = KubernetesProvider::new(config);
let manager = ServiceManager::with_provider(k8s_provider);
```

## Built-in Service Providers

### SystemdProvider
- **Platform**: Linux with systemd
- **Configuration**: `service_name` (optional, defaults to "palworld.service")
- **Commands**: `systemctl start/stop/kill`

### LaunchdProvider
- **Platform**: macOS
- **Configuration**: `service_name` (required, path to .plist file)
- **Commands**: `launchctl load/unload`

### WindowsServiceProvider
- **Platform**: Windows
- **Configuration**: `service_name` (optional, defaults to "PalWorldServer")
- **Commands**: `sc start/stop` with `taskkill` fallback

### PowerShellProvider
- **Platform**: Windows (PowerShell)
- **Configuration**: `start_command`, `stop_command`, `force_stop_command` (optional)
- **Commands**: User-defined PowerShell scripts

### CustomScriptProvider
- **Platform**: Universal
- **Configuration**: `start_command`, `stop_command`, `force_stop_command` (optional)
- **Commands**: User-defined shell commands

## Creating Custom Service Providers

### Step 1: Implement the ServiceProvider Trait

```rust
use async_trait::async_trait;
use palconnect::service_providers::{ServiceProvider, ServiceError};

#[derive(Debug, Clone)]
pub struct DockerProvider {
    container_name: String,
}

impl DockerProvider {
    pub fn new(container_name: String) -> Self {
        Self { container_name }
    }
}

#[async_trait]
impl ServiceProvider for DockerProvider {
    async fn start(&self) -> Result<String, ServiceError> {
        // Implementation
        use std::process::Command;
        
        let output = Command::new("docker")
            .arg("start")
            .arg(&self.container_name)
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Docker start failed: {}", e)))?;
        
        if output.status.success() {
            Ok(format!("Container {} started", self.container_name))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ServiceError::CommandFailed(stderr.to_string()))
        }
    }

    async fn stop(&self) -> Result<String, ServiceError> {
        // Similar implementation for stop
    }

    async fn force_stop(&self) -> Result<String, ServiceError> {
        // Docker kill implementation
    }

    fn provider_name(&self) -> &'static str {
        "docker"
    }

    fn is_available(&self) -> bool {
        std::process::Command::new("docker").arg("--version").output().is_ok()
    }
}
```

### Step 2: Use Your Custom Provider

```rust
use palconnect::ServiceManager;

// Create your custom provider
let docker_provider = DockerProvider::new("palworld-server".to_string());

// Create a service manager with your provider
let service_manager = ServiceManager::with_provider(docker_provider);

// Use normally
service_manager.start_service().await?;
```

### Step 3: Integration with Configuration (Advanced)

To integrate with PalConnect's configuration system, you would need to:

1. Add your service type to the `ServiceType` enum
2. Update the `ServiceManager::new()` method to handle your service type
3. Add configuration fields to `ServerManagement` struct

## Error Handling

The `ServiceError` enum provides structured error types:

```rust
pub enum ServiceError {
    CommandFailed(String),
    UnsupportedOperation(String),
    ConfigurationError(String),
}
```

Custom providers should use appropriate error types:

```rust
// For command execution failures
Err(ServiceError::CommandFailed("docker command failed".to_string()))

// For missing configuration
Err(ServiceError::ConfigurationError("container_name is required".to_string()))

// For unsupported operations
Err(ServiceError::UnsupportedOperation("restart not supported".to_string()))
```

## Testing Custom Providers

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_docker_provider_availability() {
        let provider = DockerProvider::new("test".to_string());
        
        // Test availability check
        if provider.is_available() {
            println!("Docker is available");
        } else {
            println!("Docker is not available, skipping Docker tests");
            return;
        }
        
        // Test provider functionality...
    }
    
    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockServiceProvider { should_fail: false };
        let result = provider.start().await;
        assert!(result.is_ok());
    }
}
```

## Examples

See `examples/custom_service_providers.rs` for complete implementations of:

- **DockerComposeProvider**: Manages Docker Compose services
- **PM2Provider**: Manages Node.js applications via PM2
- **KubernetesProvider**: Manages Kubernetes deployments

These examples demonstrate real-world service provider implementations that you can use as templates for your own providers.

## Migration from Previous Implementation

The trait-based implementation is fully backward compatible. Existing configurations continue to work without changes. The main differences are:

- **Better error handling**: More specific error types and messages
- **Improved testability**: Easy to mock for unit tests
- **Enhanced extensibility**: Users can add custom providers
- **Cleaner code structure**: Each provider is self-contained

## Performance Considerations

- Service providers are created once at startup and reused
- All operations are async for maximum flexibility
- Command execution uses `std::process::Command` for reliability
- Provider availability is checked at startup but can be re-checked anytime

## Future Enhancements

The trait-based architecture enables future enhancements like:

- **Health checks**: Additional trait methods for service health monitoring
- **Metrics collection**: Integration with monitoring systems
- **Configuration validation**: Compile-time validation of service configurations
- **Dynamic provider loading**: Runtime loading of service providers from plugins
- **Service discovery**: Automatic detection of available service providers
