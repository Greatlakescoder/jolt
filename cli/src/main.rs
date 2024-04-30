use clap::{arg, Command};
use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
};
use sys_tools::component_service;
use sys_tools::file_service;
use sys_tools::log_service;

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
        .subcommand(
            Command::new("kill-task")
                .about("Return System information")
                .arg(arg!(-p --pid <PID> "process id to kill").required(true)),
        )
}

struct AppState {
    channel_sender: Arc<Sender<u64>>,
    total: Mutex<u64>,
}

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("search", sub_matches)) => {
            let pattern = sub_matches.get_one::<String>("PATTERN").expect("required");
            let path = sub_matches.get_one::<String>("DIR").unwrap();
            let resp = file_service::grep(
                file_service::GrepRequest {
                    path: &path,
                    search_term: &pattern,
                    show_full_path: true,
                },
                Arc::new(Mutex::new(Vec::new())),
            )
            .unwrap();
            // We need to derefernece here because we want what the mutex guard is pointing to
            let data_vault = resp.lock().unwrap();
            println!("{:?}",*data_vault);
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
            let (tx, rx) = channel();
            let app_state = Arc::new(AppState {
                channel_sender: Arc::new(tx),
                total: Mutex::new(0),
            });
            let db = app_state.clone();
            let resp = file_service::find_largest_files(
                &path,
                Arc::new(Mutex::new(Vec::new())),
                db.channel_sender.clone(),
            ).unwrap();
            let data_vault = resp.lock().unwrap();
            let file_total = app_state.total.lock().unwrap();
            print!("{:?}",*data_vault);
            print!("{:?}",*file_total);
        }
        Some(("show-errors", sub_matches)) => {
            let file_count = sub_matches
                .get_one::<String>("COUNT")
                .expect("defaulted in clap");
            let file_count = file_count
                .parse::<usize>()
                .expect("COUNT must be a valid integer");
            log_service::read_sys_log(file_count);
        }
        // Some(("search-logs", sub_matches)) => {
        //     let pattern = sub_matches.get_one::<String>("PATTERN").expect("required");
        //     log_service::search(pattern);
        // }
        Some(("diagnose", sub_matches)) => {
            component_service::scan_running_proccess();
            component_service::get_network_information();
            component_service::get_system_memory();
        }
        Some(("kill-task", sub_matches)) => {
            let pid = sub_matches
                .get_one::<String>("PID")
                .expect("required in clap");
            let pid = pid.parse::<usize>().expect("PID must be a valid integer");
            component_service::kill_process(1);
        }
        _ => unreachable!(),
    }
}
