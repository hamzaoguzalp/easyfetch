use sysinfo::{System, Components, Disks, Networks};

pub enum MetricLevel {
    Ok,
    Warn,
    Crit,
    Unknown,
}

pub struct MetricRow {
    pub label: String,
    pub value: String,
    pub level: MetricLevel,
    pub progress: Option<f64>,
}

impl MetricRow {
    pub fn new(label: String, value: String, level: MetricLevel) -> Self {
        MetricRow {
            label,
            value,
            level,
            progress: None,
        }
    }

    pub fn with_progress(mut self, progress: f64) -> Self {
        self.progress = Some(progress);
        self
    }
}

pub type MonitorBox = Vec<MetricRow>;

pub enum MonitorType {
    Sys(SystemInfo),
    Mem(Box<MemoryInfo>),
    Comp(ComponentInfo),
    Disk(DiskInfo),
    Net(NetworkInfo),
}

pub fn initalize_monitors() -> Vec<MonitorType> {
    let system_info = SystemInfo::initalize_data();
    let memory_info = MemoryInfo::initalize_data();
    let component_info = ComponentInfo::initalize_data();
    let disk_info = DiskInfo::initalize_data();
    let network_info = NetworkInfo::initalize_data();

    vec![
        MonitorType::Sys(system_info),
        MonitorType::Mem(Box::new(memory_info)),
        MonitorType::Comp(component_info),
        MonitorType::Disk(disk_info),
        MonitorType::Net(network_info),
    ]
}

pub fn update_monitors(monitors: &mut Vec<MonitorType>) {
    for monitor in monitors {
        match monitor {
            MonitorType::Sys(info) => info.update_data(),
            MonitorType::Mem(info) => info.update_data(),
            MonitorType::Comp(info) => info.update_data(),
            MonitorType::Disk(info) => info.update_data(),
            MonitorType::Net(info) => info.update_data(),
        }
    }
}

pub fn fetch_all_data(monitors: &mut Vec<MonitorType>) -> Vec<(String, MonitorBox)> {
    let mut data = Vec::new();
    for monitor in monitors {
        match monitor {
            MonitorType::Sys(info) => data.push((info.title().to_string(), info.fetch_data())),
            MonitorType::Mem(info) => data.push((info.title().to_string(), info.fetch_data())),
            MonitorType::Comp(info) => data.push((info.title().to_string(), info.fetch_data())),
            MonitorType::Disk(info) => data.push((info.title().to_string(), info.fetch_data())),
            MonitorType::Net(info) => data.push((info.title().to_string(), info.fetch_data())),
        }
    }
    data
}

pub trait SystemMonitor {
    fn initalize_data() -> Self;
    fn fetch_data(&mut self) -> MonitorBox;
    fn update_data(&mut self);
    fn title(&self) -> &str;
}

pub struct SystemInfo {
    system_name: Option<String>,
    kernel_name: Option<String>,
    os_version: Option<String>,
    host_name: Option<String>,
}

impl SystemMonitor for SystemInfo {
    fn initalize_data() -> Self {
        SystemInfo {
            system_name: System::name(),
            kernel_name: System::kernel_version(),
            os_version: System::os_version(),
            host_name: System::host_name(),
        }
    }

    fn fetch_data(&mut self) -> MonitorBox {
        let check_data = |label: &str, data: &Option<String>| {
            if let Some(value) = data {
                MetricRow::new(label.to_string(), value.to_string(), MetricLevel::Ok)
            } else {
                MetricRow::new(label.to_string(), "Unknown".to_string(), MetricLevel::Unknown)
            }
        };
        vec![
            check_data("System Name :", &self.system_name),
            check_data("Kernel Name :", &self.kernel_name),
            check_data("OS Version  :", &self.os_version),
            check_data("Host Name   :", &self.host_name),
        ]
    }

    fn update_data(&mut self) { }

    fn title(&self) -> &str {
        " System Info "
    }
}

pub struct MemoryInfo {
    total_memory: u64,
    used_memory: u64,
    total_swap: u64,
    used_swap: u64,
    system: System,
}

impl SystemMonitor for MemoryInfo {
    fn initalize_data() -> Self {
        let mut sys = System::new();
        sys.refresh_memory();
        MemoryInfo {
            total_memory: sys.total_memory(),
            used_memory: sys.used_memory(),
            total_swap: sys.total_swap(),
            used_swap: sys.used_swap(),
            system: sys,
        }
    }

    fn fetch_data(&mut self) -> MonitorBox {
        self.update_data();
        let check_data = |label: &str, value, is_total, progress: Option<f64>| {
            if value == 0 && is_total {
                MetricRow::new(label.to_string(), "Unknown".to_string(), MetricLevel::Unknown)
            } else {
                let mut row = MetricRow::new(label.to_string(), convert_bytes_to_human_readable(value), MetricLevel::Ok);
                if let Some(p) = progress {
                    row = row.with_progress(p);
                }
                row
            }
        };

        let memory_percentage = if self.total_memory > 0 {
            Some((self.used_memory as f64 / self.total_memory as f64) * 100.0)
        } else {
            None
        };
        
        let swap_percentage = if self.total_swap > 0 {
            Some((self.used_swap as f64 / self.total_swap as f64) * 100.0)
        } else {
            None
        };

        vec![
            check_data("Total Memory :", self.total_memory, true  , None),
            check_data("Used Memory  :", self.used_memory,  false , memory_percentage),
            check_data("Total Swap   :", self.total_swap,   true  , None),
            check_data("Used Swap    :", self.used_swap,    false , swap_percentage),
        ]
    }

    fn update_data(&mut self) {
        self.system.refresh_memory();
        self.total_memory = self.system.total_memory();
        self.used_memory = self.system.used_memory();
        self.total_swap = self.system.total_swap();
        self.used_swap = self.system.used_swap();
    }

    fn title(&self) -> &str {
        " Memory & Swap "
    }
}

pub struct ComponentInfo {
    components: Components,
}

impl SystemMonitor for ComponentInfo {
    fn initalize_data() -> Self {
        ComponentInfo {
            components: Components::new_with_refreshed_list(),
        }
    }

    fn fetch_data(&mut self) -> MonitorBox {
        self.update_data();
        self.components
            .iter()
            .map(|component| {
                let temperature = component.temperature().unwrap_or_default();
                let temperature_text = format!("{:.0}°C", temperature);

                let level = if temperature >= 80.0 {
                    MetricLevel::Crit
                } else if temperature >= 60.0 {
                    MetricLevel::Warn
                } else {
                    MetricLevel::Ok
                };
                MetricRow::new(component.label().to_string(), temperature_text, level)
            })
            .collect()
    }
    
    fn update_data(&mut self) {
        self.components.refresh(true);
    }

    fn title(&self) -> &str {
        " Temperatures "
    }
}

pub struct DiskInfo {
    disks: Disks
}

impl SystemMonitor for DiskInfo {
    fn initalize_data() -> Self {
        DiskInfo {
            disks: Disks::new_with_refreshed_list(),
        }
    }

    fn fetch_data(&mut self) -> MonitorBox {
        self.update_data();

        self.disks
            .iter()
            .map(|disk| {
                let available_space_str = convert_bytes_to_human_readable(disk.available_space());
                let total_space_str = convert_bytes_to_human_readable(disk.total_space());
                
                let free_disk_percentage = if disk.total_space() > 0 {
                    (disk.available_space() as f64 / disk.total_space() as f64) * 100.0
                } else {
                    0.0
                };
                
                let level = if disk.total_space() == 0 {
                    MetricLevel::Unknown
                } else if free_disk_percentage <= 10.0 {
                    MetricLevel::Crit
                } else if free_disk_percentage <= 30.0 {
                    MetricLevel::Warn
                } else {
                    MetricLevel::Ok
                };

                let mut row = MetricRow::new(
                    disk.name().to_string_lossy().to_string(),
                    format!("{} free of {}", available_space_str, total_space_str),
                    level,
                );
                
                if disk.total_space() > 0 {
                    row = row.with_progress(100.0 - free_disk_percentage);
                }
                row
            })
            .collect()
    }

    fn update_data(&mut self) {
        self.disks.refresh(true);
    }

    fn title(&self) -> &str {
        " Disks "
    }
}

pub struct NetworkInfo {
    networks: Networks
}

impl SystemMonitor for NetworkInfo {
    fn initalize_data() -> Self {
        NetworkInfo {
            networks: Networks::new_with_refreshed_list(),
        }
    }

    fn fetch_data(&mut self) -> MonitorBox {
        self.update_data();

        let mut network_rows = Vec::new();

        for (interface_name, data) in self.networks.iter() {
            if data.total_received() == 0 && data.total_transmitted() == 0 {
                continue;
            }

            let speed_value = format!(
                "{:>10}/s ↓ | {:>10}/s ↑",
                convert_bytes_to_human_readable(data.received()),
                convert_bytes_to_human_readable(data.transmitted())
            );

            let total_value = format!(
                "{:>10}/s ↓ | {:>10}/s ↑",
                convert_bytes_to_human_readable(data.total_received()),
                convert_bytes_to_human_readable(data.total_transmitted())
            );
            
            network_rows.push(MetricRow::new( 
                format!("{:>5} Speed :", interface_name),
                speed_value, 
                MetricLevel::Ok, 
            ));

            network_rows.push(MetricRow::new(
                format!("{:>5} Total :", interface_name),
                total_value,
                MetricLevel::Unknown,
            ));
        }
        network_rows
    }  
    
    fn update_data(&mut self) {
        self.networks.refresh(true);
    }

    fn title(&self) -> &str {
        " Network "
    }
}

fn convert_bytes_to_human_readable(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
