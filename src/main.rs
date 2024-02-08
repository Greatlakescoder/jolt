use clap::{arg, Command};
#[macro_use]
extern crate prettytable;
mod component_service;
mod file_service;
mod log_service;
mod table_builder;

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
            log_service::read_sys_log(file_count);
        }
        Some(("search-logs", sub_matches)) => {
            let pattern = sub_matches.get_one::<String>("PATTERN").expect("required");
            log_service::search(pattern);
        }
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
            component_service::kill_process(pid);
        }
        _ => unreachable!(),
    }
}
