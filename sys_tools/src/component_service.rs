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

pub fn get_system_memory() {
    let mut sys = System::new_all();

    sys.refresh_all();
    println!("total memory: {} Mb", sys.total_memory() / 1024 / 1024);
    println!("used memory : {} Mb", sys.used_memory() / 1024 / 1024);
    println!("total swap  : {} Mb", sys.total_swap() / 1024 / 1024);
    println!("used swap   : {} Mb", sys.used_swap() / 1024 / 1024);
}

pub fn get_system_information() -> anyhow::Result<()> {
    let mut sys = System::new_all();
    sys.refresh_all();
    println!("System name:             {:?}", System::name());
    println!("System kernel version:   {:?}", System::kernel_version());
    println!("System OS version:       {:?}", System::os_version());
    println!("System host name:        {:?}", System::host_name());
    println!("System uptime:        {:?}", System::uptime());
    // Number of CPUs:
    println!("NB CPUs: {}", sys.cpus().len());
    println!("total memory: {} Mb", sys.total_memory() / 1024 / 1024);
    println!("Cpu Arch {}", System::cpu_arch().unwrap());
    println!("total memory: {} Mb", sys.total_memory() / 1024 / 1024);
    Ok(())

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

// Direct way for checking info on process
//
//
//
// #[cfg(target_os = "linux")]
// pub fn scan_running_proccess() -> Vec<JoltOutput> {
//     if cfg!(target_os = "linux") {
//         let output = OsCommand::new("ps")
//             .arg("-eo")
//             .arg("user,pid,%cpu,%mem,vsz,rss,tty,stat,start,time,command")
//             .arg("--sort=-user,-%cpu")
//             .arg("r")
//             .output()
//             .expect("failed to execute process");
//         // Cow stands for clone on write and is a smart pointer that allows you to have either a reference to some data or the data itself.
//         let output = String::from_utf8_lossy(&output.stdout);
//         let records = output.split("\n").collect::<Vec<&str>>();

//         let mut output: Vec<JoltOutput> = vec![];
//         for record in records.iter().skip(1) {
//             let fields: Vec<&str> = record.split_whitespace().collect();
//             if fields.len() < 11 {
//                 continue;
//             }
//             let mut new_record = JoltOutput {
//                 user: fields[0].to_string(),
//                 pid: fields[1].to_string(),
//                 cpu: fields[2].to_string(),
//                 mem: fields[3].parse().unwrap_or(0),
//                 vsz: fields[4].to_string(),
//                 rss: fields[5].to_string(),
//                 tty: fields[6].to_string(),
//                 stat: fields[7].to_string(),
//                 start: fields[8].to_string(),
//                 time: fields[9].to_string(),
//                 command: fields[10].to_string(),
//             };
//             new_record.command.truncate(15);
//             output.push(new_record);
//         }
//         return output;
//     } else {
//         let output = OsCommand::new("WMIC")
//             .arg("path")
//             .arg("Win32_PerfFormattedData_PerfProc_Process")
//             .arg("get")
//             .arg("Name,IDProcess,PercentProcessorTime,WorkingSetPrivate")
//             .output()
//             .expect("failed to execute process");
//         let output = String::from_utf8_lossy(&output.stdout);
//         let records = output.split("\n").collect::<Vec<&str>>();
//         let mut processes = Vec::new();

//         for record in records.iter().skip(2) {
//             // Skip the first 2 lines of output as they don't contain task info
//             let fields: Vec<&str> = record.split_whitespace().collect();
//             if fields.len() < 4 {
//                 continue;
//             }
//             let new_record = JoltOutput {
//                 user: fields[0].to_string(),         // Image Name
//                 pid: fields[1].to_string(),          // PID
//                 cpu: fields[2].to_string(),          // CPU Usage
//                 mem: fields[3].parse().unwrap_or(0), // Memory Usage
//                 command: "".to_string(),
//                 rss: "".to_string(),
//                 start: "".to_string(),
//                 stat: "".to_string(),
//                 time: "".to_string(),
//                 tty: "".to_string(),
//                 vsz: "".to_string(),
//             };
//             processes.push(new_record);
//         }

//         // Sort by CPU usage
//         processes.sort_by(|a, b| {
//             b.cpu
//                 .parse::<f64>()
//                 .unwrap_or(0.0)
//                 .partial_cmp(&a.cpu.parse::<f64>().unwrap_or(0.0))
//                 .unwrap_or(std::cmp::Ordering::Equal)
//         });
//         return processes;
//     };
// }
