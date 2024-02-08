use serde::Deserialize;
use std::process::Command as OsCommand;
use crate::file_service;

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
    vec![Data {
        name: String::from("No Data"),
        value: String::from("No Data"),
    }]
}

fn default_event_data_value() -> EventData {
    EventData {
        data: default_vec_value(),
    }
}

#[cfg(target_os = "windows")]
pub fn read_sys_log(count: usize) {
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

pub fn read_win_sys_log(search_term: String) {
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
pub fn read_sys_log(count: usize) {
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

pub fn search(search_term: &str) {
    #[cfg(target_os = "linux")]
    {
        file_service::grep("/var/log", search_term)
    }

    #[cfg_attr(rustc_dummy, cfg(target_os = "windows"))]
    {
        read_win_sys_log(search_term.to_string());
    }
}
