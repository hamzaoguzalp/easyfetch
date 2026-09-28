use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::fs;
use std::mem::MaybeUninit;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    Mem(MemoryInfo),
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
        MonitorType::Mem(memory_info),
        MonitorType::Comp(component_info),
        MonitorType::Disk(disk_info),
        MonitorType::Net(network_info),
    ]
}

pub fn update_monitors(monitors: &mut [MonitorType]) {
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

pub fn fetch_all_data(monitors: &mut [MonitorType]) -> Vec<(String, MonitorBox)> {
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

// -------------------------------------------------------------
// 1. SystemInfo (/proc/sys/kernel/osrelease, hostname, /etc/os-release)
// -------------------------------------------------------------
pub struct SystemInfo {
    system_name: Option<String>,
    kernel_name: Option<String>,
    os_version: Option<String>,
    host_name: Option<String>,
}

impl SystemMonitor for SystemInfo {
    fn initalize_data() -> Self {
        let (system_name, kernel_name, os_version, host_name) = read_system_info();
        SystemInfo {
            system_name,
            kernel_name,
            os_version,
            host_name,
        }
    }

    fn fetch_data(&mut self) -> MonitorBox {
        let check_data = |label: &str, data: &Option<String>| {
            if let Some(value) = data {
                MetricRow::new(label.to_string(), value.to_string(), MetricLevel::Ok)
            } else {
                MetricRow::new(
                    label.to_string(),
                    "Unknown".to_string(),
                    MetricLevel::Unknown,
                )
            }
        };
        vec![
            check_data("OS", &self.system_name),
            check_data("Kernel", &self.kernel_name),
            check_data("Version", &self.os_version),
            check_data("Host", &self.host_name),
        ]
    }

    fn update_data(&mut self) {}

    fn title(&self) -> &str {
        " 󰍹 System Info "
    }
}

fn read_system_info() -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let mut system_name = None;
    let mut os_version = None;

    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("PRETTY_NAME=") {
                system_name = Some(val.trim_matches('"').to_string());
            } else if system_name.is_none() && line.starts_with("NAME=") {
                system_name = Some(line["NAME=".len()..].trim_matches('"').to_string());
            } else if let Some(val) = line.strip_prefix("VERSION_ID=") {
                os_version = Some(val.trim_matches('"').to_string());
            } else if os_version.is_none() && line.starts_with("BUILD_ID=") {
                os_version = Some(line["BUILD_ID=".len()..].trim_matches('"').to_string());
            }
        }
    }

    if system_name.is_none() {
        system_name = fs::read_to_string("/proc/sys/kernel/ostype")
            .ok()
            .map(|s| s.trim().to_string());
    }

    let kernel_name = fs::read_to_string("/proc/sys/kernel/osrelease")
        .ok()
        .map(|s| s.trim().to_string());

    let host_name = fs::read_to_string("/proc/sys/kernel/hostname")
        .ok()
        .map(|s| s.trim().to_string());

    (system_name, kernel_name, os_version, host_name)
}

// -------------------------------------------------------------
// 2. MemoryInfo (/proc/meminfo)
// -------------------------------------------------------------
pub struct MemoryInfo {
    total_memory: u64,
    used_memory: u64,
    total_swap: u64,
    used_swap: u64,
}

impl SystemMonitor for MemoryInfo {
    fn initalize_data() -> Self {
        let (total_memory, used_memory, total_swap, used_swap) = read_meminfo();
        MemoryInfo {
            total_memory,
            used_memory,
            total_swap,
            used_swap,
        }
    }

    fn fetch_data(&mut self) -> MonitorBox {
        self.update_data();
        let mut rows = Vec::new();

        if self.total_memory > 0 {
            let pct = (self.used_memory as f64 / self.total_memory as f64) * 100.0;
            let val = format!(
                "{} / {} ({:.0}%)",
                convert_bytes_to_human_readable(self.used_memory),
                convert_bytes_to_human_readable(self.total_memory),
                pct
            );
            let level = if pct >= 85.0 {
                MetricLevel::Crit
            } else if pct >= 65.0 {
                MetricLevel::Warn
            } else {
                MetricLevel::Ok
            };
            rows.push(MetricRow::new("RAM".to_string(), val, level).with_progress(pct));
        }

        if self.total_swap > 0 {
            let pct = (self.used_swap as f64 / self.total_swap as f64) * 100.0;
            let val = format!(
                "{} / {} ({:.0}%)",
                convert_bytes_to_human_readable(self.used_swap),
                convert_bytes_to_human_readable(self.total_swap),
                pct
            );
            let level = if pct >= 85.0 {
                MetricLevel::Crit
            } else if pct >= 65.0 {
                MetricLevel::Warn
            } else {
                MetricLevel::Ok
            };
            rows.push(MetricRow::new("Swap".to_string(), val, level).with_progress(pct));
        }

        rows
    }

    fn update_data(&mut self) {
        let (total_memory, used_memory, total_swap, used_swap) = read_meminfo();
        self.total_memory = total_memory;
        self.used_memory = used_memory;
        self.total_swap = total_swap;
        self.used_swap = used_swap;
    }

    fn title(&self) -> &str {
        " 󰘚 Memory "
    }
}

fn read_meminfo() -> (u64, u64, u64, u64) {
    let mut total_mem = 0;
    let mut avail_mem = 0;
    let mut total_swap = 0;
    let mut free_swap = 0;

    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("MemTotal:") {
                total_mem = parse_kb(rest);
            } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
                avail_mem = parse_kb(rest);
            } else if let Some(rest) = line.strip_prefix("SwapTotal:") {
                total_swap = parse_kb(rest);
            } else if let Some(rest) = line.strip_prefix("SwapFree:") {
                free_swap = parse_kb(rest);
            }
        }
    }

    let used_mem = total_mem.saturating_sub(avail_mem);
    let used_swap = total_swap.saturating_sub(free_swap);
    (total_mem, used_mem, total_swap, used_swap)
}

fn parse_kb(s: &str) -> u64 {
    s.split_whitespace()
        .next()
        .and_then(|v| v.parse::<u64>().ok())
        .map(|kb| kb.saturating_mul(1024))
        .unwrap_or(0)
}

// -------------------------------------------------------------
// 3. ComponentInfo (/sys/class/hwmon)
// -------------------------------------------------------------
pub struct ComponentInfo;

impl SystemMonitor for ComponentInfo {
    fn initalize_data() -> Self {
        ComponentInfo
    }

    fn fetch_data(&mut self) -> MonitorBox {
        let mut rows = Vec::new();
        let mut seen_spd_count = 0;

        if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
            let mut hwmon_dirs: Vec<_> = entries.filter_map(|e| e.ok().map(|d| d.path())).collect();
            hwmon_dirs.sort();

            for dir in hwmon_dirs {
                let name = fs::read_to_string(dir.join("name"))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                if let Ok(files) = fs::read_dir(&dir) {
                    let mut temp_inputs: Vec<_> = files
                        .filter_map(|f| f.ok().map(|d| d.path()))
                        .filter(|p| {
                            p.file_name()
                                .and_then(|n| n.to_str())
                                .map(|s| s.starts_with("temp") && s.ends_with("_input"))
                                .unwrap_or(false)
                        })
                        .collect();
                    temp_inputs.sort();

                    for input_path in temp_inputs {
                        let content = match fs::read_to_string(&input_path) {
                            Ok(c) => c,
                            Err(_) => continue,
                        };
                        let m_celsius: i64 = match content.trim().parse() {
                            Ok(val) => val,
                            Err(_) => continue,
                        };

                        if m_celsius <= -100_000 || m_celsius > 200_000 {
                            continue;
                        }

                        let temp = m_celsius as f64 / 1000.0;
                        let stem = input_path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("temp1")
                            .replace("_input", "");

                        let label_path = dir.join(format!("{stem}_label"));
                        let sub_label = fs::read_to_string(label_path)
                            .map(|s| s.trim().to_string())
                            .unwrap_or_else(|_| stem);

                        let raw_label = format!("{name} {sub_label}");
                        let mut friendly = format_friendly_sensor_name(&raw_label);

                        if friendly.starts_with("Memory SPD") {
                            seen_spd_count += 1;
                            friendly = format!("Memory SPD {seen_spd_count}");
                        }

                        let temperature_text = format!("{:.0}°C", temp);
                        let level = if temp >= 80.0 {
                            MetricLevel::Crit
                        } else if temp >= 60.0 {
                            MetricLevel::Warn
                        } else {
                            MetricLevel::Ok
                        };
                        let pct = temp.clamp(0.0, 100.0);
                        rows.push(
                            MetricRow::new(friendly, temperature_text, level).with_progress(pct),
                        );
                    }
                }
            }
        }

        rows.sort_by_key(|r| sensor_priority(&r.label));
        rows
    }

    fn update_data(&mut self) {}

    fn title(&self) -> &str {
        " 󰔏 Temperatures "
    }
}

pub fn format_friendly_sensor_name(raw: &str) -> String {
    let lower = raw.to_lowercase();

    // CPU (AMD Ryzen, Intel Core)
    if lower.starts_with("k10temp") {
        if lower.contains("tctl") {
            return "CPU (Tctl)".to_string();
        } else if lower.contains("tdie") {
            return "CPU (Tdie)".to_string();
        }
        return "CPU".to_string();
    }
    if lower.starts_with("coretemp") {
        if lower.contains("package") {
            return "CPU (Package)".to_string();
        }
        return "CPU".to_string();
    }
    if lower.contains("cpu") {
        return "CPU".to_string();
    }

    // GPU (AMD Radeon, Nvidia, Nouveau)
    if lower.starts_with("amdgpu") {
        if lower.contains("edge") {
            return "GPU (Edge)".to_string();
        } else if lower.contains("junction") {
            return "GPU (Junction)".to_string();
        } else if lower.contains("mem") {
            return "GPU (VRAM)".to_string();
        }
        return "GPU".to_string();
    }
    if lower.contains("nvidia") || lower.contains("nouveau") {
        return "GPU".to_string();
    }

    // NVMe / SSD
    if lower.starts_with("nvme") {
        if lower.contains("composite") {
            return "NVMe (Composite)".to_string();
        } else if lower.contains("sensor 1") {
            return "NVMe (Sensor 1)".to_string();
        } else if lower.contains("sensor 2") {
            return "NVMe (Sensor 2)".to_string();
        }
        return "NVMe SSD".to_string();
    }

    // Wi-Fi / Wireless
    if lower.starts_with("mt79")
        || lower.starts_with("iwl")
        || lower.starts_with("ath")
        || lower.starts_with("rtw")
        || lower.contains("wifi")
        || lower.contains("wlan")
    {
        return "Wi-Fi".to_string();
    }

    // RAM / DDR5 SPD Hub
    if lower.starts_with("spd5118")
        || lower.starts_with("jc42")
        || lower.contains("dram")
        || lower.contains("spd")
    {
        if let Some(digit) = lower.chars().last().filter(|c| c.is_ascii_digit()) {
            return format!("Memory SPD {digit}");
        }
        return "Memory (SPD)".to_string();
    }

    // Motherboard / ACPI
    if lower.starts_with("acpitz") {
        if let Some(digit) = lower.chars().last().filter(|c| c.is_ascii_digit()) {
            return format!("Motherboard {digit}");
        }
        return "Motherboard".to_string();
    }

    raw.to_string()
}

pub fn sensor_priority(name: &str) -> usize {
    if name.starts_with("CPU") {
        0
    } else if name.starts_with("GPU") {
        1
    } else if name.starts_with("NVMe") {
        2
    } else if name.starts_with("Memory") {
        3
    } else if name.starts_with("Wi-Fi") {
        4
    } else {
        5
    }
}

// -------------------------------------------------------------
// 4. DiskInfo (/proc/mounts + libc::statvfs)
// -------------------------------------------------------------
pub struct DiskInfo;

impl SystemMonitor for DiskInfo {
    fn initalize_data() -> Self {
        DiskInfo
    }

    fn fetch_data(&mut self) -> MonitorBox {
        let mut seen = HashSet::new();
        let mut seen_fs_sizes = HashSet::new();
        let mut rows = Vec::new();

        if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
            for line in mounts.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 2 {
                    continue;
                }
                let dev = parts[0];
                let mount = parts[1].to_string();

                if !dev.starts_with("/dev") {
                    continue;
                }

                if mount.starts_with("/proc")
                    || mount.starts_with("/sys")
                    || mount.starts_with("/dev")
                    || mount.starts_with("/run")
                    || !seen.insert(mount.clone())
                {
                    continue;
                }

                if let Some((total, available)) = get_disk_space(&mount) {
                    if total == 0 {
                        continue;
                    }

                    if mount != "/" && !seen_fs_sizes.insert((total, available)) {
                        continue;
                    }
                    if mount == "/" {
                        seen_fs_sizes.insert((total, available));
                    }

                    let used = total.saturating_sub(available);
                    let pct = (used as f64 / total as f64) * 100.0;

                    let level = if pct >= 90.0 {
                        MetricLevel::Crit
                    } else if pct >= 75.0 {
                        MetricLevel::Warn
                    } else {
                        MetricLevel::Ok
                    };

                    let val = format!(
                        "{} / {} ({:.0}%)",
                        convert_bytes_to_human_readable(used),
                        convert_bytes_to_human_readable(total),
                        pct
                    );

                    rows.push(MetricRow::new(mount, val, level).with_progress(pct));
                }
            }
        }

        if rows.is_empty() {
            rows.push(MetricRow::new(
                "Storage".to_string(),
                "No disks found".to_string(),
                MetricLevel::Unknown,
            ));
        }

        rows
    }

    fn update_data(&mut self) {}

    fn title(&self) -> &str {
        " 󰋊 Storage "
    }
}

fn get_disk_space(mount_point: &str) -> Option<(u64, u64)> {
    let c_path = CString::new(mount_point).ok()?;
    unsafe {
        let mut stat = MaybeUninit::<libc::statvfs>::uninit();
        if libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) == 0 {
            let stat = stat.assume_init();
            let block_size = stat.f_frsize;
            let total = stat.f_blocks * block_size;
            let available = stat.f_bavail * block_size;
            Some((total, available))
        } else {
            None
        }
    }
}

// -------------------------------------------------------------
// 5. NetworkInfo (/proc/net/dev)
// -------------------------------------------------------------
struct NetSnapshot {
    rx_bytes: u64,
    tx_bytes: u64,
    time: Instant,
}

pub struct NetworkInfo {
    last_snapshots: HashMap<String, NetSnapshot>,
}

impl SystemMonitor for NetworkInfo {
    fn initalize_data() -> Self {
        let mut info = NetworkInfo {
            last_snapshots: HashMap::new(),
        };
        info.update_data();
        info
    }

    fn fetch_data(&mut self) -> MonitorBox {
        self.update_data();
        let mut network_rows = Vec::new();
        let now = Instant::now();

        let current_devs = read_net_dev();
        let has_real_iface = current_devs.iter().any(|(name, _, _)| {
            name.starts_with("wl") || name.starts_with("en") || name.starts_with("eth")
        });

        for (iface, rx_total, tx_total) in current_devs {
            if has_real_iface
                && (iface == "lo"
                    || iface.starts_with("virbr")
                    || iface.starts_with("docker")
                    || iface.starts_with("veth")
                    || iface.starts_with("br-"))
            {
                continue;
            }

            if rx_total == 0 && tx_total == 0 {
                continue;
            }

            let (rx_speed, tx_speed) = if let Some(last) = self.last_snapshots.get(&iface) {
                let elapsed = now.duration_since(last.time).as_secs_f64();
                if elapsed > 0.05 {
                    let rx_spd =
                        ((rx_total.saturating_sub(last.rx_bytes)) as f64 / elapsed).round() as u64;
                    let tx_spd =
                        ((tx_total.saturating_sub(last.tx_bytes)) as f64 / elapsed).round() as u64;
                    (rx_spd, tx_spd)
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };

            self.last_snapshots.insert(
                iface.clone(),
                NetSnapshot {
                    rx_bytes: rx_total,
                    tx_bytes: tx_total,
                    time: now,
                },
            );

            let speed_val = format!(
                "{:>8}/s ↓  |  {:>8}/s ↑",
                convert_bytes_to_human_readable(rx_speed),
                convert_bytes_to_human_readable(tx_speed),
            );

            let total_val = format!(
                "{:>10} ↓  |  {:>10} ↑",
                convert_bytes_to_human_readable(rx_total),
                convert_bytes_to_human_readable(tx_total),
            );

            network_rows.push(MetricRow::new(
                format!("{} Speed", iface),
                speed_val,
                MetricLevel::Ok,
            ));
            network_rows.push(MetricRow::new(
                format!("{} Total", iface),
                total_val,
                MetricLevel::Unknown,
            ));
        }

        if network_rows.is_empty() {
            network_rows.push(MetricRow::new(
                "Network".to_string(),
                "Disconnected".to_string(),
                MetricLevel::Unknown,
            ));
        }

        network_rows
    }

    fn update_data(&mut self) {}

    fn title(&self) -> &str {
        " 󰖩 Network "
    }
}

fn read_net_dev() -> Vec<(String, u64, u64)> {
    let mut list = Vec::new();
    if let Ok(content) = fs::read_to_string("/proc/net/dev") {
        for line in content.lines() {
            if let Some((iface_part, stats_part)) = line.split_once(':') {
                let iface = iface_part.trim().to_string();
                let numbers: Vec<u64> = stats_part
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if numbers.len() >= 9 {
                    let rx = numbers[0];
                    let tx = numbers[8];
                    list.push((iface, rx, tx));
                }
            }
        }
    }
    list
}

fn convert_bytes_to_human_readable(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_friendly_sensor_names() {
        assert_eq!(format_friendly_sensor_name("k10temp Tctl"), "CPU (Tctl)");
        assert_eq!(
            format_friendly_sensor_name("coretemp Package id 0"),
            "CPU (Package)"
        );
        assert_eq!(format_friendly_sensor_name("amdgpu edge"), "GPU (Edge)");
        assert_eq!(
            format_friendly_sensor_name("nvme Composite WD PC SN810 SDCPNRY-1T00-1006"),
            "NVMe (Composite)"
        );
        assert_eq!(format_friendly_sensor_name("mt7921_phy0 temp1"), "Wi-Fi");
        assert_eq!(format_friendly_sensor_name("spd5118 temp1"), "Memory SPD 1");
        assert_eq!(format_friendly_sensor_name("acpitz temp1"), "Motherboard 1");
    }

    #[test]
    fn test_sensor_priority() {
        assert!(sensor_priority("CPU (Tctl)") < sensor_priority("GPU (Edge)"));
        assert!(sensor_priority("GPU (Edge)") < sensor_priority("NVMe (Composite)"));
        assert!(sensor_priority("NVMe (Composite)") < sensor_priority("Memory SPD 1"));
    }

    #[test]
    fn test_read_meminfo() {
        let (total, _, _, _) = read_meminfo();
        assert!(total > 0);
    }

    #[test]
    fn test_read_system_info() {
        let (name, kernel, _, host) = read_system_info();
        assert!(name.is_some());
        assert!(kernel.is_some());
        assert!(host.is_some());
    }

    #[test]
    fn test_read_net_dev() {
        let devs = read_net_dev();
        // At least loopback should be present
        assert!(!devs.is_empty());
    }
}
