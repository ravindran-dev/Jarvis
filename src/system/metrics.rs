use anyhow::Result;
use log::warn;
use sysinfo::{Disks, Networks, System};

/// CPU usage information
#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub usage: f32,
    pub per_core: Vec<f32>,
}

/// Memory usage information
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    #[allow(dead_code)]
    pub available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

/// Disk usage information
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total: u64,
    pub used: u64,
    #[allow(dead_code)]
    pub available: u64,
}

/// Network usage information
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub received: u64,
    pub sent: u64,
    pub rx_rate: u64,
    pub tx_rate: u64,
}

/// System metrics collector
pub struct SystemMetrics {
    system: System,
    disks: Disks,
    networks: Networks,
    previous_net_rx: u64,
    previous_net_tx: u64,
}

impl SystemMetrics {
    /// Create a new SystemMetrics instance
    pub fn new() -> Result<Self> {
        let mut system = System::new_all();
        system.refresh_all();

        let disks = Disks::new_with_refreshed_list();
        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh();

        Ok(Self {
            system,
            disks,
            networks,
            previous_net_rx: 0,
            previous_net_tx: 0,
        })
    }

    /// Update all system metrics
    pub fn update(&mut self) -> Result<()> {
        self.system.refresh_cpu();
        self.system.refresh_memory();
        self.disks.refresh();
        self.networks.refresh();

        Ok(())
    }

    /// Get current CPU usage information
    pub fn get_cpu_info(&self) -> CpuInfo {
        let usage = self.system.global_cpu_info().cpu_usage();

        let per_core: Vec<f32> = self
            .system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage())
            .collect();

        CpuInfo { usage, per_core }
    }

    /// Get current memory usage information
    pub fn get_memory_info(&self) -> MemoryInfo {
        let total = self.system.total_memory();
        let available = self.system.available_memory();
        let used = total.saturating_sub(available);
        let swap_total = self.system.total_swap();
        let swap_used = swap_total.saturating_sub(self.system.free_swap());

        MemoryInfo {
            total,
            used,
            available,
            swap_total,
            swap_used,
        }
    }

    /// Get disk usage information for all mounted disks
    pub fn get_disk_info(&self) -> Vec<DiskInfo> {
        self.disks
            .iter()
            .map(|disk| {
                let total = disk.total_space();
                let available = disk.available_space();
                let used = total.saturating_sub(available);

                DiskInfo {
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    total,
                    used,
                    available,
                }
            })
            .collect()
    }

    /// Get network usage information
    pub fn get_network_info(&mut self) -> NetworkInfo {
        let mut total_received = 0u64;
        let mut total_sent = 0u64;

        for (_interface_name, network) in &self.networks {
            total_received += network.total_received();
            total_sent += network.total_transmitted();
        }

        let rx_rate = total_received.saturating_sub(self.previous_net_rx);
        let tx_rate = total_sent.saturating_sub(self.previous_net_tx);

        self.previous_net_rx = total_received;
        self.previous_net_tx = total_sent;

        NetworkInfo {
            received: total_received,
            sent: total_sent,
            rx_rate,
            tx_rate,
        }
    }

    /// Get system temperature (if available)
    pub fn get_temperature(&self) -> Option<f32> {
        self.read_temperature_from_procfs()
    }

    /// Attempt to read temperature from /sys/class/thermal
    fn read_temperature_from_procfs(&self) -> Option<f32> {
        use std::fs;

        for i in 0..10 {
            let path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(temp_millidegrees) = content.trim().parse::<i32>() {
                    let temp_celsius = temp_millidegrees as f32 / 1000.0;
                    if temp_celsius > 0.0 && temp_celsius < 150.0 {
                        return Some(temp_celsius);
                    }
                }
            }
        }

        warn!("Temperature sensors not available");
        None
    }
}
