use std::collections::HashMap;
use sysinfo::{CpuExt, System, SystemExt, NetworkExt, ProcessExt};


#[derive(Debug)]
pub struct Size {
    pub size: f32,
    pub unit: char
}

impl Size {
    pub fn convent_from(size: u64) -> Self {
        let mut size_ = size as f32 / 8_f32;
        let mut level = 1_u8;
        loop {
            if size_ < 1.0 || level >= 5 {
                break
            }
            size_ = size_ / 1024.0;
            level += 1
        }
        Self {
            size: size_,
            unit: match level {
                1 => 'k',
                2 => 'M',
                3 => 'G',
                4 => 'T',
                5 => 'P',
                _ => unreachable!()
            }
        }
    }

    pub fn format(self) -> String{
        format!("{:.2}{}", self.size, self.unit)
    }
}


/// Memory Info
#[derive(Debug)]
pub struct  AppMemoryInfo {
    pub total_memory: Size,
    pub used_memory: Size,
    pub total_swap: Size,
    pub used_swap: Size,
    pub swap_usage: f32,
    pub memory_usage: f32
}


/// CPU Info
#[derive(Debug)]
pub struct  AppCPUInfo {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
}


/// Network Info
#[derive(Debug)]
pub struct  AppNetworkInfo {
    pub received_list: Vec<u64>,
    pub transmitted_list: Vec<u64>,

    pub detail: HashMap<String, [u64; 2]>,

    pub received: u64,
    pub transmitted: u64,
}


/// Sysinfo
#[derive(Debug)]
pub struct AppSystemInfo {
    sys: System,

    // System Info
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,

    pub global_cpu_usage: f32,

    pub memory_info: AppMemoryInfo,

    pub network_info: AppNetworkInfo

}

impl Default for AppSystemInfo {
    fn default() -> Self {
        let sys = System::new_all();
        let hostname = sys.host_name().unwrap_or("Unknown".to_string());
        let os_name = sys.name().unwrap_or("Unknown".to_string());
        let os_version = sys.os_version().unwrap_or("Unknown".to_string());
        let kernel_version = sys.kernel_version().unwrap_or("Unknown".to_string());

        let total_memory = sys.total_memory();
        let total_swap = sys.total_swap();

        Self {
            sys,
            hostname,
            os_name,
            os_version,
            kernel_version,
            global_cpu_usage: 0.0,
            memory_info: AppMemoryInfo {
                total_memory: Size::convent_from(total_memory),
                used_memory: Size { size: 0.0, unit: ' ' },
                total_swap: Size::convent_from(total_swap),
                used_swap: Size { size: 0.0, unit: ' ' },
                swap_usage: 0.0,
                memory_usage: 0.0,
            },
            network_info: AppNetworkInfo{
                received_list: vec![0; 300],
                transmitted_list: vec![0; 300],
                detail: HashMap::<String, [u64; 2]>::default(),
                received: 0,
                transmitted: 0,
            }
        }
    }
}


impl AppSystemInfo {

    pub fn refresh_global_cpu_usage(&mut self) {
        self.sys.refresh_cpu();
        self.global_cpu_usage = self.sys.global_cpu_info().cpu_usage()
    }

    pub fn refresh_memory_info(&mut self) {
        self.sys.refresh_memory();
        let memory_usage = (self.sys.used_memory() as f32 / self.sys.total_memory() as f32) * 100.0;

        if self.sys.total_swap() == 0 {
            self.memory_info.used_memory = Size::convent_from(self.sys.used_memory());
            self.memory_info.memory_usage = memory_usage;
        } else {
            self.memory_info.used_memory = Size::convent_from(self.sys.used_memory());
            self.memory_info.used_swap = Size::convent_from(self.sys.used_swap());
            self.memory_info.memory_usage = memory_usage;
            self.memory_info.swap_usage = self.memory_info.used_swap.size as f32 / self.memory_info.total_swap.size as f32
        }

    }

    pub fn refresh_network_info(&mut self) -> &AppNetworkInfo {
        self.sys.refresh_networks();

        let mut received = 0_u64;
        let mut transmitted = 0_u64;

        self.sys.networks().into_iter().for_each(|(interface, data)| {
            let rec = data.received();
            let tx = data.transmitted();

            self.network_info.detail.insert(interface.to_string(), [rec, tx]);

            received += data.received();
            self.network_info.received_list.pop();
            self.network_info.received_list.insert(0, rec);

            transmitted += data.transmitted();
            self.network_info.transmitted_list.pop();
            self.network_info.transmitted_list.insert(0, tx);
        });

        self.network_info.received = received;
        self.network_info.transmitted = transmitted;

        &self.network_info
    }

    pub fn get_process_info(&mut self) -> (Vec<&str>, Vec<Vec<String>>) {
        let process_columns = vec![
            "CPU%",
            "MEM%",
            "PID",
            // "USER",
            "VIRT",
            "RES",
            "TIME+",
            // "PRI",
            // "NI",
            // "THR",
            "S",
            "R/s",
            "W/s",
            "Command",
        ];
        let mut process_data = vec![];
        self.sys.refresh_processes();

        for (pid, process) in self.sys.processes() {
            process_data.push(vec![
               format!("{:.1}", process.cpu_usage()),        // CPU%
               format!("{:.1}", process.memory() / self.sys.total_memory()),        // MEM%
               format!("{}", pid.to_string()),        // PID
               // format!("{}", self.sys.get_user_by_id(process.user_id().unwrap()).unwrap().name()),        // USER
               format!("{}", Size::convent_from(process.virtual_memory()).format()),        // VIRT
               format!("{}", Size::convent_from(process.memory()).format()),        // RES
               format!("{}", process.run_time()),        // TIME+
               // format!(pid.to_string()),        // PRI
               // format!(pid.to_string()),        // NI
               // format!(pid.to_string()),        // THR
               format!("{}", process.status().to_string()),        // S
               format!("{}", Size::convent_from(process.disk_usage().read_bytes).format()),        // R/s
               format!("{}", Size::convent_from(process.disk_usage().written_bytes).format()),        // W/s
               format!("{}", process.cmd().join(" ")),        // Command
            ])
        }
        return (process_columns, process_data);
    }

    pub fn refresh_all(&mut self) {
        self.refresh_global_cpu_usage();

        self.refresh_memory_info();

        self.refresh_network_info();

    }

}