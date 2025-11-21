//! Example custom service provider implementation
//! 
//! This file demonstrates how to create a custom service provider that integrates
//! with PalConnect's trait-based service management system.

use async_trait::async_trait;
use palconnect::service_providers::{ServiceProvider, ServiceError};
use log::{debug, warn};
use std::process::Command;

/// Example: Docker Compose service provider
/// 
/// This provider manages PalWorld servers running in Docker Compose setups.
#[derive(Debug, Clone)]
pub struct DockerComposeProvider {
    compose_file: String,
    service_name: String,
    project_name: Option<String>,
}

impl DockerComposeProvider {
    pub fn new(compose_file: String, service_name: String, project_name: Option<String>) -> Self {
        Self {
            compose_file,
            service_name,
            project_name,
        }
    }

    fn build_command(&self, action: &str) -> Command {
        let mut cmd = Command::new("docker");
        cmd.arg("compose");
        
        if let Some(project) = &self.project_name {
            cmd.arg("-p").arg(project);
        }
        
        cmd.arg("-f").arg(&self.compose_file);
        cmd.arg(action);
        
        if action != "down" {
            cmd.arg(&self.service_name);
        }
        
        cmd
    }
}

#[async_trait]
impl ServiceProvider for DockerComposeProvider {
    async fn start(&self) -> Result<String, ServiceError> {
        debug!("Starting Docker Compose service: {}", self.service_name);
        
        let output = self.build_command("up")
            .arg("-d") // detached mode
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute docker compose: {}", e)))?;
        
        if output.status.success() {
            Ok(format!("Docker Compose service '{}' started successfully", self.service_name))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ServiceError::CommandFailed(format!("Docker Compose start failed: {}", stderr)))
        }
    }

    async fn stop(&self) -> Result<String, ServiceError> {
        debug!("Stopping Docker Compose service: {}", self.service_name);
        
        let output = self.build_command("stop")
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute docker compose: {}", e)))?;
        
        if output.status.success() {
            Ok(format!("Docker Compose service '{}' stopped successfully", self.service_name))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ServiceError::CommandFailed(format!("Docker Compose stop failed: {}", stderr)))
        }
    }

    async fn force_stop(&self) -> Result<String, ServiceError> {
        debug!("Force stopping Docker Compose service: {}", self.service_name);
        
        // Try to kill the containers
        let output = self.build_command("kill")
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute docker compose: {}", e)))?;
        
        if output.status.success() {
            Ok(format!("Docker Compose service '{}' force stopped successfully", self.service_name))
        } else {
            warn!("Docker Compose kill failed, trying graceful stop");
            self.stop().await
        }
    }

    fn provider_name(&self) -> &'static str {
        "docker_compose"
    }

    fn is_available(&self) -> bool {
        Command::new("docker").arg("compose").arg("version").output().is_ok()
    }
}

/// Example: PM2 (Process Manager 2) provider
/// 
/// This provider manages PalWorld servers using PM2, a popular Node.js process manager
/// that can also manage other types of applications.
#[derive(Debug, Clone)]
pub struct PM2Provider {
    app_name: String,
}

impl PM2Provider {
    pub fn new(app_name: String) -> Self {
        Self { app_name }
    }
}

#[async_trait]
impl ServiceProvider for PM2Provider {
    async fn start(&self) -> Result<String, ServiceError> {
        debug!("Starting PM2 app: {}", self.app_name);
        
        let output = Command::new("pm2")
            .arg("start")
            .arg(&self.app_name)
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute pm2: {}", e)))?;
        
        if output.status.success() {
            Ok(format!("PM2 app '{}' started successfully", self.app_name))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ServiceError::CommandFailed(format!("PM2 start failed: {}", stderr)))
        }
    }

    async fn stop(&self) -> Result<String, ServiceError> {
        debug!("Stopping PM2 app: {}", self.app_name);
        
        let output = Command::new("pm2")
            .arg("stop")
            .arg(&self.app_name)
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute pm2: {}", e)))?;
        
        if output.status.success() {
            Ok(format!("PM2 app '{}' stopped successfully", self.app_name))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ServiceError::CommandFailed(format!("PM2 stop failed: {}", stderr)))
        }
    }

    async fn force_stop(&self) -> Result<String, ServiceError> {
        debug!("Force stopping PM2 app: {}", self.app_name);
        
        let output = Command::new("pm2")
            .arg("delete")
            .arg(&self.app_name)
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute pm2: {}", e)))?;
        
        if output.status.success() {
            Ok(format!("PM2 app '{}' force stopped and deleted", self.app_name))
        } else {
            warn!("PM2 delete failed, trying graceful stop");
            self.stop().await
        }
    }

    fn provider_name(&self) -> &'static str {
        "pm2"
    }

    fn is_available(&self) -> bool {
        Command::new("pm2").arg("--version").output().is_ok()
    }
}

/// Example: Kubernetes provider
/// 
/// This provider manages PalWorld servers running as Kubernetes deployments.
#[derive(Debug, Clone)]
pub struct KubernetesProvider {
    namespace: String,
    deployment_name: String,
    kubeconfig: Option<String>,
}

impl KubernetesProvider {
    pub fn new(namespace: String, deployment_name: String, kubeconfig: Option<String>) -> Self {
        Self {
            namespace,
            deployment_name,
            kubeconfig,
        }
    }

    fn build_kubectl_command(&self, action: &str) -> Command {
        let mut cmd = Command::new("kubectl");
        
        if let Some(config) = &self.kubeconfig {
            cmd.arg("--kubeconfig").arg(config);
        }
        
        cmd.arg("-n").arg(&self.namespace);
        cmd.arg(action);
        
        cmd
    }
}

#[async_trait]
impl ServiceProvider for KubernetesProvider {
    async fn start(&self) -> Result<String, ServiceError> {
        debug!("Starting Kubernetes deployment: {}", self.deployment_name);
        
        let output = self.build_kubectl_command("scale")
            .arg("deployment")
            .arg(&self.deployment_name)
            .arg("--replicas=1")
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute kubectl: {}", e)))?;
        
        if output.status.success() {
            Ok(format!("Kubernetes deployment '{}' started successfully", self.deployment_name))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ServiceError::CommandFailed(format!("Kubernetes scale up failed: {}", stderr)))
        }
    }

    async fn stop(&self) -> Result<String, ServiceError> {
        debug!("Stopping Kubernetes deployment: {}", self.deployment_name);
        
        let output = self.build_kubectl_command("scale")
            .arg("deployment")
            .arg(&self.deployment_name)
            .arg("--replicas=0")
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute kubectl: {}", e)))?;
        
        if output.status.success() {
            Ok(format!("Kubernetes deployment '{}' stopped successfully", self.deployment_name))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ServiceError::CommandFailed(format!("Kubernetes scale down failed: {}", stderr)))
        }
    }

    async fn force_stop(&self) -> Result<String, ServiceError> {
        debug!("Force stopping Kubernetes deployment: {}", self.deployment_name);
        
        // Delete all pods for immediate termination
        let output = self.build_kubectl_command("delete")
            .arg("pods")
            .arg("-l")
            .arg(format!("app={}", self.deployment_name))
            .arg("--force")
            .arg("--grace-period=0")
            .output()
            .map_err(|e| ServiceError::CommandFailed(format!("Failed to execute kubectl: {}", e)))?;
        
        if output.status.success() {
            // Also scale down to 0
            let _ = self.stop().await;
            Ok(format!("Kubernetes deployment '{}' force stopped", self.deployment_name))
        } else {
            warn!("Kubernetes force delete failed, trying graceful stop");
            self.stop().await
        }
    }

    fn provider_name(&self) -> &'static str {
        "kubernetes"
    }

    fn is_available(&self) -> bool {
        Command::new("kubectl").arg("version").arg("--client").output().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_provider_availability() {
        let docker_provider = DockerComposeProvider::new(
            "docker-compose.yml".to_string(),
            "palworld".to_string(),
            None,
        );
        
        let pm2_provider = PM2Provider::new("palworld-app".to_string());
        
        let k8s_provider = KubernetesProvider::new(
            "gaming".to_string(),
            "palworld-server".to_string(),
            None,
        );
        
        // These tests will depend on what's installed on the system
        println!("Docker Compose available: {}", docker_provider.is_available());
        println!("PM2 available: {}", pm2_provider.is_available());
        println!("Kubernetes available: {}", k8s_provider.is_available());
    }
}
