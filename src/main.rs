use clap::{arg, Command};
use core::fmt;
use csv::ReaderBuilder;
use prettytable::{Cell, Row, Table};
use serde::Deserialize;
use std::ffi::OsString;
use std::path::Display;
use std::path::PathBuf;
use std::process::Command as OsCommand;
use std::thread;
use std::time::Duration;
use std::{collections::HashMap, sync::atomic::AtomicBool, sync::atomic::Ordering, sync::Arc};
use sysinfo::{Components, Disks, Networks, System};
use xml::reader::{EventReader, XmlEvent};

use clap::Parser;

mod file_service;
#[macro_use]
extern crate prettytable;

// /// Simple program to greet a person
// #[derive(Parser, Debug)]
// #[command(author, version, about, long_about = None)]
// struct Args {
//     #[arg(short,long,default_value_t = String::from("./test_files"))]
//     path: String,
//     #[arg(short,long,default_value_t = 20)]
//     file_count: usize,
// }

#[derive(Debug, Deserialize)]
struct EventRecord {
    #[serde(rename = "System")]
    system: SystemInfo,
    #[serde(rename = "EventData", default = "default_event_data_value")]
    event_data: EventData,
}

#[derive(Debug, Deserialize)]
struct SystemInfo {
    #[serde(rename = "EventID")]
    event_id: String,
    #[serde(rename = "TimeCreated")]
    time_created: TimeCreated,
    #[serde(rename = "Level")]
    level: String,
    #[serde(rename = "Provider")]
    provider: Provider,
    #[serde(rename = "EventRecordID")]
    event_record_id: String,
    
}

#[derive(Debug, Deserialize)]
struct Provider {
    #[serde(rename = "Name")]
    name: String,
}

#[derive(Debug, Deserialize)]
struct TimeCreated {
    #[serde(rename = "SystemTime")]
    system_time: String,
}

#[derive(Debug, Deserialize)]
struct EventData {
    #[serde(rename = "Data", default = "default_vec_value")]
    data: Vec<Data>,
}

#[derive(Debug, Deserialize)]
struct Data {
    #[serde(rename = "name", default = "default_value")]
    name: String,
    #[serde(rename = "$value", default = "default_value")]
    value: String,
}
fn default_value() -> String {
    "".to_string()
}

fn default_vec_value() -> Vec<Data> {
    vec![Data{
        name: String::from("No Data"),
        value: String::from("No Data")
    }]
}

fn default_event_data_value() -> EventData {
    EventData{data: default_vec_value()}
}

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

fn get_system_memory() {
    let mut sys = System::new_all();

    sys.refresh_all();
    println!("total memory: {} Mb", sys.total_memory() / 1024 / 1024);
    println!("used memory : {} Mb", sys.used_memory() / 1024 / 1024);
    println!("total swap  : {} Mb", sys.total_swap() / 1024 / 1024);
    println!("used swap   : {} Mb", sys.used_swap() / 1024 / 1024);
}

fn get_system_information() {
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

fn get_network_information() {
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

#[cfg(target_os = "linux")]
fn scan_running_proccess() {
    let output = OsCommand::new("ps")
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

#[cfg(target_os = "windows")]
fn read_sys_log(count: usize) {
    let output = OsCommand::new("wevtutil")
        .arg("qe")
        .arg("System")
        .arg("/q:*[System[(Level=2)]]")
        .arg(format!("/c:{}", count))
        .arg("/rd:true")
        .arg("/f:xml")
        .output()
        .expect("Failed to execute command");

    let output = std::str::from_utf8(&output.stdout).expect("Not UTF8");
    // Parse the XML and convert it to JSON
    let events: Vec<EventRecord> = serde_xml_rs::from_str(output).unwrap();
    for event in events {
        println!("{:?}", event);
    }
}


fn read_win_sys_log(search_term: String) {
    let output = OsCommand::new("wevtutil")
        .arg("qe")
        .arg("System")
        .arg("/rd:true")
        .arg("/f:xml")
        .output()
        .expect("Failed to execute command");

    let output = std::str::from_utf8(&output.stdout).expect("Not UTF8");
    // Parse the XML and convert it to JSON
    let events: Vec<EventRecord> = serde_xml_rs::from_str(output).unwrap();
    for event in events {
        for row in &event.event_data.data {
            if row.value.contains(&search_term) {
                println!("{:?}", &event);
                break;
            }
        }
        
    }
}

#[cfg(target_os = "linux")]
fn read_sys_log(count: usize) {
    let output = OsCommand::new("journalctl")
        .arg("-p")
        .arg("err") // Only show error messages
        .arg("-n") // Number of journal entries to show
        .arg(count.to_string()) // Show the last 5 entries
        .output()
        .expect("Failed to execute command");

    let output = std::str::from_utf8(&output.stdout).expect("Not UTF8");
    println!("{}", output);
}

fn search(search_term: &str) {
    #[cfg(target_os = "linux")]
    {
        file_service::grep("/var/log", search_term)
    }

    #[cfg_attr(rustc_dummy, cfg(target_os = "windows"))]
    {
        read_win_sys_log(search_term.to_string());
    }
}

fn cli() -> Command {
    Command::new("jolt")
        .about("Diagnostic tool to help give your computer that extra jolt")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("search")
                .about("Search for a file")
                .arg(arg!(-p <PATTERN> "pattern to search for").required(true))
                .arg(
                    arg!(-d <DIR>"directory to search")
                        .required(false)
                        .default_value("./test_files"),
                ),
        )
        .subcommand(
            Command::new("search-logs")
                .about("Search logs with a pattern")
                .arg(arg!(-p <PATTERN> "pattern to search for").required(true)),
        )
        .subcommand(
            Command::new("space-finder")
                .about("Find largest files ")
                .arg(arg!(-d <DIR> "directory to search").default_value("./test_files"))
                .arg(
                    arg!(-c <COUNT> "number of files to return")
                        .required(false)
                        .default_value("20"),
                ),
        )
        .subcommand(
            Command::new("show-errors").about("Show recent errors").arg(
                arg!(-c <COUNT> "number of files to return")
                    .required(false)
                    .default_value("20"),
            ),
        )
        .subcommand(Command::new("diagnose").about("Return System information"))
}

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("search", sub_matches)) => {
            let pattern = sub_matches.get_one::<String>("PATTERN").expect("required");
            let path = sub_matches.get_one::<String>("DIR").unwrap();
            file_service::grep(path, &pattern);
        }
        Some(("space-finder", sub_matches)) => {
            let path = sub_matches
                .get_one::<String>("DIR")
                .map(|s| s.as_str())
                .expect("defaulted in clap");
            let file_count = sub_matches
                .get_one::<String>("COUNT")
                .expect("defaulted in clap");
            let file_count = file_count
                .parse::<usize>()
                .expect("COUNT must be a valid integer");
            let mut folder = file_service::find_largest_files(
                path,
                file_service::Folder::new(String::from("Large Files"), file_count),
            );
            folder.sort_files();
            folder.print_files();
        }
        Some(("show-errors", sub_matches)) => {
            let file_count = sub_matches
                .get_one::<String>("COUNT")
                .expect("defaulted in clap");
            let file_count = file_count
                .parse::<usize>()
                .expect("COUNT must be a valid integer");
            read_sys_log(file_count);
        }
        Some(("search-logs", sub_matches)) => {
            let pattern = sub_matches.get_one::<String>("PATTERN").expect("required");
            search(pattern);
        }
        Some(("diagnose", sub_matches)) => {
            scan_running_proccess();
            get_network_information();
            get_system_memory();
        }
        _ => unreachable!(),
    }
}
