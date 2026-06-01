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

/// Represents the OS service manager used to control the PalWorld server process.
///
/// - `Systemd` — uses `systemctl` (Linux with systemd)
/// - `InitD`   — uses `/etc/init.d/<name>` (Linux without systemd / SysV init)
/// - `None`    — no OS-level management; only the PalWorld REST API is used
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceManager {
    Systemd,
    InitD,
    None,
}

impl ServiceManager {
    /// Parse the service manager type from a string (case-insensitive).
    /// Accepts "systemd", "initd" / "sysvinit" / "init.d", or "none".
    /// Defaults to `Systemd` for unrecognised values.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "systemd" => ServiceManager::Systemd,
            "initd" | "sysvinit" | "init.d" => ServiceManager::InitD,
            "none" => ServiceManager::None,
            other => {
                log::warn!("⚠️ Unknown service manager '{}', defaulting to systemd", other);
                ServiceManager::Systemd
            }
        }
    }

    /// Start the named OS service.
    ///
    /// Returns `Ok(exit_code)` on success, `Err` if the command could not be spawned.
    #[cfg(unix)]
    pub fn start(&self, service_name: &str) -> std::io::Result<i32> {
        let status = match self {
            ServiceManager::Systemd => {
                log::info!("▶️ Starting '{}' via systemctl", service_name);
                std::process::Command::new("systemctl")
                    .args(["start", service_name])
                    .status()?
            }
            ServiceManager::InitD => {
                log::info!("▶️ Starting '{}' via /etc/init.d", service_name);
                std::process::Command::new("/etc/init.d/".to_string() + service_name)
                    .arg("start")
                    .status()?
            }
            ServiceManager::None => {
                log::warn!("⚠️ Service manager is 'none' — cannot start '{}' via OS service unit", service_name);
                return Ok(-1);
            }
        };
        Ok(status.code().unwrap_or(-1))
    }

    /// Gracefully stop the named OS service.
    ///
    /// Returns `Ok(exit_code)` on success, `Err` if the command could not be spawned.
    #[cfg(unix)]
    pub fn stop(&self, service_name: &str) -> std::io::Result<i32> {
        let status = match self {
            ServiceManager::Systemd => {
                log::info!("⏹️ Stopping '{}' via systemctl", service_name);
                std::process::Command::new("systemctl")
                    .args(["stop", service_name])
                    .status()?
            }
            ServiceManager::InitD => {
                log::info!("⏹️ Stopping '{}' via /etc/init.d", service_name);
                std::process::Command::new("/etc/init.d/".to_string() + service_name)
                    .arg("stop")
                    .status()?
            }
            ServiceManager::None => {
                log::warn!("⚠️ Service manager is 'none' — cannot stop '{}' via OS service unit", service_name);
                return Ok(-1);
            }
        };
        Ok(status.code().unwrap_or(-1))
    }

    /// Force-kill the named OS service immediately (SIGKILL / equivalent).
    ///
    /// - systemd: `systemctl kill --signal=SIGKILL <name>`
    /// - initd:   `service <name> stop` then a SIGKILL on any remaining PID via the PID file
    ///            (`/var/run/<name>.pid` or `/run/<name>.pid`).
    ///
    /// Returns `Ok(exit_code)` on success, `Err` if a command could not be spawned.
    #[cfg(unix)]
    pub fn force_stop(&self, service_name: &str) -> std::io::Result<i32> {
        match self {
            ServiceManager::Systemd => {
                log::info!("🔥 Force-killing '{}' via systemctl (SIGKILL)", service_name);
                let status = std::process::Command::new("systemctl")
                    .args(["kill", "--signal=SIGKILL", service_name])
                    .status()?;
                Ok(status.code().unwrap_or(-1))
            }
            ServiceManager::InitD => {
                log::info!("🔥 Force-killing '{}' via init.d + SIGKILL", service_name);
                // First attempt a normal stop so the init script cleans up
                let _ = std::process::Command::new("/etc/init.d/".to_string() + service_name)
                    .arg("stop")
                    .status();
                // Then SIGKILL any surviving process from the PID file
                let pid_candidates = [
                    format!("/var/run/{}.pid", service_name),
                    format!("/run/{}.pid", service_name),
                ];
                for pid_path in &pid_candidates {
                    if let Ok(contents) = std::fs::read_to_string(pid_path) {
                        if let Ok(pid) = contents.trim().parse::<u32>() {
                            log::info!("🔥 Sending SIGKILL to PID {} (from {})", pid, pid_path);
                            let _ = std::process::Command::new("kill")
                                .args(["-9", &pid.to_string()])
                                .status();
                        }
                    }
                }
                Ok(0)
            }
            ServiceManager::None => {
                log::warn!("⚠️ Service manager is 'none' — cannot force-kill '{}' via OS service unit", service_name);
                Ok(-1)
            }
        }
    }

    /// Returns a human-readable label for use in Discord messages.
    pub fn label(&self) -> &'static str {
        match self {
            ServiceManager::Systemd => "systemd",
            ServiceManager::InitD => "init.d",
            ServiceManager::None => "none",
        }
    }

    /// Returns whether this manager is capable of OS-level process control.
    pub fn is_capable(&self) -> bool {
        !matches!(self, ServiceManager::None)
    }
}
