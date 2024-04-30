use core::fmt;

use serde::{Deserialize, Serialize};

use sysinfo::{Networks, System};

use psutil::process::processes;

#[derive(Serialize, Deserialize)]
pub struct JoltOutput {
    user: String,
    pid: String,
    cpu: String,
    mem: i32,
    time: String,
    command: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SystemInformation {
    pub name: String,
    pub os_version: String,
    pub host_name: String,
    pub uptime: u64,
    pub total_cpus: u64,
    pub total_memory: u64,
    pub cpu_arch: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SystemMemory {
    total_memory: u64,
    used_memory: u64,
    total_swap: u64,
    used_swap: u64,
}

impl fmt::Display for SystemInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Name {} \n 
            Os Version {} \n
            Hostname {} \n
            uptime {} \n
            total cpus {} \n
            total memory {} \n
            cpu arch {} \n",
            self.name,
            self.os_version,
            self.host_name,
            self.uptime,
            self.total_cpus,
            self.total_memory,
            self.cpu_arch
        )
    }
}

impl fmt::Display for JoltOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Effective User Id {} \n 
            Process Id {} \n
            Cpu Usage {} \n
            Memory Usage {} \n
            Cumulative CPU Time {} \n
            Command {} \n",
            self.user, self.pid, self.cpu, self.mem, self.time, self.command
        )
    }
}

pub fn get_system_memory() -> SystemMemory {
    let mut sys = System::new_all();

    sys.refresh_all();
    return SystemMemory {
        total_memory: sys.total_memory() / 1024 / 1024,
        used_memory: sys.used_memory() / 1024 / 1024,
        total_swap: sys.total_swap() / 1024 / 1024,
        used_swap: sys.used_swap() / 1024 / 1024,
    };
}

pub fn get_system_information() -> anyhow::Result<SystemInformation> {
    let mut sys = System::new_all();
    sys.refresh_all();
    Ok(SystemInformation {
        cpu_arch: psutil::host::info().architecture().to_string(),
        host_name: psutil::host::info().hostname().to_string(),
        os_version: psutil::host::info().operating_system().to_string(),
        name: System::name().unwrap(),
        uptime: psutil::host::uptime().unwrap().as_secs(),
        total_cpus: psutil::cpu::cpu_count(),
        total_memory: psutil::memory::virtual_memory().unwrap().available(),
    })
}

pub fn get_network_information() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let networks = Networks::new_with_refreshed_list();
    let users = sysinfo::Users::new();
    for user in users.list() {
        println!("{} is in {} groups", user.name(), user.groups().len());
    }
    println!("=> networks:");
    for (interface_name, data) in &networks {
        println!(
            "{interface_name}: {}/{} B  Mac Address: {}, Packets {}/{}",
            data.received(),
            data.transmitted(),
            data.mac_address(),
            data.packets_received(),
            data.packets_transmitted()
        );
    }
}

pub fn kill_process(pid: u32) -> anyhow::Result<()> {
    let current_process = psutil::process::Process::new(pid)?;
    if current_process.is_running() {
        current_process.kill()?;
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct CpuUsageResponse {
    pub cpus: Vec<CpuUsage>,
}
#[derive(Serialize, Deserialize)]
pub struct CpuUsage {
    pub name: String,
    pub brand: String,
    pub frequency: u64,
    pub usage: f32,
}

pub fn get_current_cpu_usage() -> CpuUsageResponse {
    println!("Get Current CPU Start");
    let mut s = System::new_with_specifics(
        sysinfo::RefreshKind::new().with_cpu(sysinfo::CpuRefreshKind::everything()),
    );

    // Wait a bit because CPU usage is based on diff.
    println!("Get Current CPU Sleep");
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    // Refresh CPUs again.
    println!("Get Current CPU Refresh");
    s.refresh_cpu();
    println!("Get Current CPU Logic");

    let mut resp: Vec<CpuUsage> = vec![];
    for cpu in s.cpus() {
        resp.push(CpuUsage {
            name: cpu.name().to_string(),
            brand: cpu.brand().to_string(),
            frequency: cpu.frequency(),
            usage: cpu.cpu_usage(),
        })
    }
    CpuUsageResponse { cpus: resp }
}

#[derive(Serialize, Deserialize)]
pub struct MemoryResponse {
    pub free_memory: u64,
    pub total_memory: u64,
}

pub fn get_memory_cpu_usage() -> MemoryResponse {
    let mut s = System::new_with_specifics(
        sysinfo::RefreshKind::new().with_memory(sysinfo::MemoryRefreshKind::everything()),
    );

    // Wait a bit because CPU usage is based on diff.
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    // Refresh Memory
    s.refresh_memory();

    MemoryResponse {
        free_memory: s.free_memory(),
        total_memory: s.total_memory(),
    }
}

pub fn scan_running_proccess() -> anyhow::Result<Vec<JoltOutput>> {
    let mut output: Vec<JoltOutput> = vec![];

    for alive_process in processes()? {
        let mut process = alive_process?;
        let name = process.pid();
        let pid = process.pid();
        let cpu = process.cpu_percent()?;
        let mem = process.memory_percent()?;
        let command = process.name()?;
        let start_time = process.create_time().as_secs().to_string();

        let new_record = JoltOutput {
            user: name.to_string(),
            pid: pid.to_string(),
            cpu: cpu.to_string(),
            mem: mem as i32,
            time: start_time,
            command,
        };

        output.push(new_record);
    }

    Ok(output)
}
