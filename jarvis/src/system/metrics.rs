use anyhow::Result;

use sysinfo::{Disks, Networks, System};

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub usage: f32,
    pub per_core: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total: u64,
    pub used: u64,
}

#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub received: u64,
    pub sent: u64,
    pub rx_rate: u64,
    pub tx_rate: u64,
}

pub struct SystemMetrics {
    system: System,
    disks: Disks,
    networks: Networks,
    previous_net_rx: u64,
    previous_net_tx: u64,
}

impl SystemMetrics {
    pub fn new() -> Result<Self> {
        let mut system = System::new_all();
        system.refresh_all();

        let disks = Disks::new_with_refreshed_list();
        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh(true);

        Ok(Self {
            system,
            disks,
            networks,
            previous_net_rx: 0,
            previous_net_tx: 0,
        })
    }

    pub fn update(&mut self) -> Result<()> {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.disks.refresh(true);
        self.networks.refresh(true);

        Ok(())
    }

    pub fn get_cpu_info(&self) -> CpuInfo {
        let usage = self.system.global_cpu_usage();

        let per_core: Vec<f32> = self
            .system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage())
            .collect();

        CpuInfo { usage, per_core }
    }

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
                }
            })
            .collect()
    }

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
}
