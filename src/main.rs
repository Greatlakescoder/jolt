use core::fmt;
use csv::ReaderBuilder;
use prettytable::{Cell, Row, Table};
use std::collections::HashMap;
use std::path::Display;
use std::process::Command;
use sysinfo::{Components, Disks, Networks, System};

#[macro_use]
extern crate prettytable;
struct JoltOutput {
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

fn get_system_preformance() {
    // Please note that we use "new_all" to ensure that all list of
    // components, network interfaces, disks and users are already
    // filled!
    let mut sys = System::new_all();

    // First we update all information of our `System` struct.
    sys.refresh_all();

    println!("=> system:");
    // RAM and swap information:
    println!("total memory: {} bytes", sys.total_memory());
    println!("used memory : {} bytes", sys.used_memory());
    println!("total swap  : {} bytes", sys.total_swap());
    println!("used swap   : {} bytes", sys.used_swap());

    // Display system information:
    println!("System name:             {:?}", System::name());
    println!("System kernel version:   {:?}", System::kernel_version());
    println!("System OS version:       {:?}", System::os_version());
    println!("System host name:        {:?}", System::host_name());

    // Number of CPUs:
    println!("NB CPUs: {}", sys.cpus().len());

    // Display processes ID, name na disk usage:
    for (pid, process) in sys.processes() {
        println!("[{pid}] {} {:?}", process.name(), process.disk_usage());
    }

    // We display all disks' information:
    println!("=> disks:");
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        println!("{disk:?}");
    }

    // Network interfaces name, data received and data transmitted:
    let networks = Networks::new_with_refreshed_list();
    println!("=> networks:");
    for (interface_name, data) in &networks {
        println!(
            "{interface_name}: {}/{} B",
            data.received(),
            data.transmitted()
        );
    }

    // Components temperature:
    let components = Components::new_with_refreshed_list();
    println!("=> components:");
    for component in &components {
        println!("{component:?}");
    }
    sys.refresh_cpu(); // Refreshing CPU information.
    for cpu in sys.cpus() {
        print!("{}% ", cpu.cpu_usage());
    }
}

#[cfg(target_os = "linux")]
fn scan_running_proccess() {
    let output = Command::new("ps")
        .arg("-eo")
        .arg("user,pid,%cpu,%mem,vsz,rss,tty,stat,start,time,command")
        .arg("--sort=-user,-%cpu")
        .output()
        .expect("failed to execute process");
    // Cow stands for clone on write and is a smart pointer that allows you to have either a reference to some data or the data itself.
    let output = String::from_utf8_lossy(&output.stdout);
    let records = output.split("\n").collect::<Vec<&str>>();
    let mut table = Table::new();
    table.add_row(row![
        "User", "PID", "%CPU", "%MEM", "VSZ", "RSS", "TTY", "STAT", "START", "TIME", "COMMAND"
    ]);

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
        table.add_row(Row::new(vec![
            Cell::new(&new_record.user),
            Cell::new(&new_record.pid),
            Cell::new(&new_record.cpu),
            Cell::new(&new_record.mem.to_string()),
            Cell::new(&new_record.vsz),
            Cell::new(&new_record.rss),
            Cell::new(&new_record.tty),
            Cell::new(&new_record.stat),
            Cell::new(&new_record.start),
            Cell::new(&new_record.time),
            Cell::new(&new_record.command),
        ]));
        // println!("{} \n ", new_record);
    }
    table.printstd();
    // println!("{:?}", records)
}

#[cfg(target_os = "windows")]
fn scan_running_proccess() {
    let output = Command::new("WMIC")
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

    let mut table = Table::new();
    table.add_row(row!["Image Name", "PID", "CPU Usage", "Memory Usage"]);

    for process in processes {
        let memory_mb = &process.mem / 1024 / 1024;
        table.add_row(Row::new(vec![
            Cell::new(&process.user),
            Cell::new(&process.pid),
            Cell::new(&process.cpu),
            Cell::new(memory_mb.to_string().as_str()),
        ]));
    }

    table.printstd();
}

fn list_file() {
    let output = Command::new("ls")
        .arg("-l")
        .arg("-a")
        .arg("-s")
        .output()
        .expect("failed to execute process");
    let output = String::from_utf8_lossy(&output.stdout);
    println!("{}", output);
}

fn main() {
    // list_file();
    get_system_preformance();
    scan_running_proccess();
}
