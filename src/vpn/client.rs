//! VPN client - wraps protonvpn-cli

use std::process::Command;

use anyhow::Context;

use super::types::{ConnectionStats, Server};
use crate::error::{AppError, AppResult};

/// VPN client for interacting with protonvpn-cli
pub struct VpnClient {
    /// Path to protonvpn-cli (default: protonvpn-cli)
    cli_path: String,
}

impl Default for VpnClient {
    fn default() -> Self {
        Self::new()
    }
}

impl VpnClient {
    pub fn new() -> Self {
        Self {
            cli_path: String::from("protonvpn-cli"),
        }
    }

    /// Connect to a server
    pub fn connect(&self, server: &str) -> AppResult<()> {
        let output = Command::new(&self.cli_path)
            .args(["-c", server])
            .output()
            .context("Failed to execute protonvpn-cli")
            .map_err(|e| AppError::ConnectionFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::ConnectionFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// Disconnect from VPN
    pub fn disconnect(&self) -> AppResult<()> {
        let output = Command::new(&self.cli_path)
            .args(["-d"])
            .output()
            .context("Failed to execute protonvpn-cli")
            .map_err(|e| AppError::ConnectionFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::ConnectionFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// List available servers
    pub fn list_servers(&self) -> AppResult<Vec<Server>> {
        let output = Command::new(&self.cli_path)
            .args(["-s"])
            .output()
            .context("Failed to execute protonvpn-cli")
            .map_err(|e| AppError::ConnectionFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::ConnectionFailed(stderr.to_string()));
        }

        // Parse output - TODO: implement actual parsing
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::debug!("Server list output: {}", stdout);

        // Placeholder - return empty list for now
        Ok(Vec::new())
    }

    /// Get connection status
    pub fn status(&self) -> AppResult<String> {
        let output = Command::new(&self.cli_path)
            .args(["-s"])
            .output()
            .context("Failed to execute protonvpn-cli")
            .map_err(|e| AppError::ConnectionFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// Get connection statistics
    pub fn stats(&self) -> AppResult<ConnectionStats> {
        // TODO: Parse actual stats from protonvpn-cli
        Ok(ConnectionStats::default())
    }
}
