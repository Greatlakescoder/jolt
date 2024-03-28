use core::fmt;
use prettytable::{Cell, Row, Table};
use serde::{Deserialize, Serialize};
use std::process::Command as OsCommand;
use sysinfo::{Components, Disks, Networks, System};

use crate::table_builder::MagicTable;

#[derive(Serialize, Deserialize)]
pub struct JoltOutput {
    user: String,
    pid: String,
    cpu: String,
    mem: i32,
    vsz: String,
    rss: String,
    tty: String,
    stat: String,
    start: String,
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
            Virtual Memory Size in KiB{} \n
            Resident Set Size in KiB {} \n
            Controlling Terminal {} \n
            Process State {} \n
            Start Time {} \n
            Cumulative CPU Time {} \n
            Command {} \n",
            self.user,
            self.pid,
            self.cpu,
            self.mem,
            self.vsz,
            self.rss,
            self.tty,
            self.stat,
            self.start,
            self.time,
            self.command
        )
    }
}

// table.add_row(row![
//     "User", "PID", "%CPU", "%MEM", "VSZ", "RSS", "TTY", "STAT", "START", "TIME", "COMMAND"
// ]);
impl MagicTable for JoltOutput {
    fn build_table(&self) -> Vec<(String, String)> {
        vec![
            ("User".to_string(), self.user.to_string()),
            ("PID".to_string(), self.pid.to_string()),
            ("%CPU".to_string(), self.cpu.to_string()),
            ("%MEM".to_string(), self.mem.to_string()),
            ("VSZ".to_string(), self.vsz.to_string()),
            ("RSS".to_string(), self.rss.to_string()),
            ("TTY".to_string(), self.tty.to_string()),
            ("STAT".to_string(), self.stat.to_string()),
            ("START".to_string(), self.start.to_string()),
            ("TIME".to_string(), self.time.to_string()),
            ("COMMAND".to_string(), self.command.to_string()),
        ]
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

pub fn get_system_information() {
    let mut sys = System::new_all();
    sys.refresh_all();
    println!("System name:             {:?}", System::name());
    println!("System kernel version:   {:?}", System::kernel_version());
    println!("System OS version:       {:?}", System::os_version());
    println!("System host name:        {:?}", System::host_name());
    // Number of CPUs:
    println!("NB CPUs: {}", sys.cpus().len());
    println!("total memory: {} Mb", sys.total_memory() / 1024 / 1024);
}

pub fn get_network_information() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let networks = Networks::new_with_refreshed_list();
    println!("=> networks:");
    for (interface_name, data) in &networks {
        println!(
            "{interface_name}: {}/{} B",
            data.received(),
            data.transmitted()
        );
    }
}

pub fn kill_process(pid: usize) {
    #[cfg(unix)]
    let status = OsCommand::new("kill")
        .arg(pid.to_string())
        .status()
        .unwrap();

    #[cfg(windows)]
    let status = OsCommand::new("taskkill")
        .arg("/F")
        .arg("/PID")
        .arg(pid.to_string())
        .status()
        .unwrap();

    if status.success() {
        println!("Successfully killed process {}", pid);
    } else {
        println!("Failed to kill process {}", pid);
    }
}

#[cfg(target_os = "linux")]
pub fn scan_running_proccess() -> Vec<JoltOutput> {

    if cfg!(target_os = "linux") {
        let output = OsCommand::new("ps")
            .arg("-eo")
            .arg("user,pid,%cpu,%mem,vsz,rss,tty,stat,start,time,command")
            .arg("--sort=-user,-%cpu")
            .output()
            .expect("failed to execute process");
        // Cow stands for clone on write and is a smart pointer that allows you to have either a reference to some data or the data itself.
        let output = String::from_utf8_lossy(&output.stdout);
        let records = output.split("\n").collect::<Vec<&str>>();

        let mut output: Vec<JoltOutput> = vec![];
        for record in records.iter().skip(1) {
            let fields: Vec<&str> = record.split_whitespace().collect();
            if fields.len() < 11 {
                continue;
            }
            let mut new_record = JoltOutput {
                user: fields[0].to_string(),
                pid: fields[1].to_string(),
                cpu: fields[2].to_string(),
                mem: fields[3].parse().unwrap_or(0),
                vsz: fields[4].to_string(),
                rss: fields[5].to_string(),
                tty: fields[6].to_string(),
                stat: fields[7].to_string(),
                start: fields[8].to_string(),
                time: fields[9].to_string(),
                command: fields[10].to_string(),
            };
            new_record.command.truncate(15);
            output.push(new_record);
        }
        return output;
    } else {
        let output = OsCommand::new("WMIC")
            .arg("path")
            .arg("Win32_PerfFormattedData_PerfProc_Process")
            .arg("get")
            .arg("Name,IDProcess,PercentProcessorTime,WorkingSetPrivate")
            .output()
            .expect("failed to execute process");
        let output = String::from_utf8_lossy(&output.stdout);
        let records = output.split("\n").collect::<Vec<&str>>();
        let mut processes = Vec::new();

        for record in records.iter().skip(2) {
            // Skip the first 2 lines of output as they don't contain task info
            let fields: Vec<&str> = record.split_whitespace().collect();
            if fields.len() < 4 {
                continue;
            }
            let new_record = JoltOutput {
                user: fields[0].to_string(),         // Image Name
                pid: fields[1].to_string(),          // PID
                cpu: fields[2].to_string(),          // CPU Usage
                mem: fields[3].parse().unwrap_or(0), // Memory Usage
                command: "".to_string(),
                rss: "".to_string(),
                start: "".to_string(),
                stat: "".to_string(),
                time: "".to_string(),
                tty: "".to_string(),
                vsz: "".to_string(),
            };
            processes.push(new_record);
        }

        // Sort by CPU usage
        processes.sort_by(|a, b| {
            b.cpu
                .parse::<f64>()
                .unwrap_or(0.0)
                .partial_cmp(&a.cpu.parse::<f64>().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        return processes;
    };
}