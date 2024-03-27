// let mut table = Table::new();

use std::vec;

// for record in records.iter().skip(1) {
//     let fields: Vec<&str> = record.split_whitespace().collect();
//     if fields.len() < 11 {
//         continue;
//     }
//     let mut new_record = JoltOutput {
//         user: fields[0].to_string(),
//         pid: fields[1].to_string(),
//         cpu: fields[2].to_string(),
//         mem: fields[3].parse().unwrap_or(0),
//         vsz: fields[4].to_string(),
//         rss: fields[5].to_string(),
//         tty: fields[6].to_string(),
//         stat: fields[7].to_string(),
//         start: fields[8].to_string(),
//         time: fields[9].to_string(),
//         command: fields[10].to_string(),
//     };
//     new_record.command.truncate(15);
//     table.add_row(Row::new(vec![
//         Cell::new(&new_record.user),
//         Cell::new(&new_record.pid),
//         Cell::new(&new_record.cpu),
//         Cell::new(&new_record.mem.to_string()),
//         Cell::new(&new_record.vsz),
//         Cell::new(&new_record.rss),
//         Cell::new(&new_record.tty),
//         Cell::new(&new_record.stat),
//         Cell::new(&new_record.start),
//         Cell::new(&new_record.time),
//         Cell::new(&new_record.command),
//     ]));
//     // println!("{} \n ", new_record);
// }
// table.printstd();

//
//
// The Goal of this file is to be able to take a struct and turn it into a table without user haveing to do
// a lot of work
//
//

pub trait MagicTable {
    fn build_table(&self) -> Vec<(String, String)>;
}

use prettytable::{Cell, Row, Table};
pub fn build_table<T: MagicTable>(item: T) -> Table {
    let mut table = Table::new();
    for (field_name, field_value) in item.build_table() {
        table.add_row(Row::new(vec![
            Cell::new(&field_name),
            Cell::new(&field_value),
        ]));
    }
    table
}
// pub struct TableBuilder {
//     table: Table,
// }

// impl TableBuilder {
//     pub fn new() -> TableBuilder {
//         TableBuilder {
//             table: Table::new(),
//         }
//     }

//     //"User", "PID", "%CPU", "%MEM", "VSZ", "RSS", "TTY", "STAT", "START", "TIME", "COMMAND"
//     pub fn create_headers(&mut self, headers: Vec<String>) {
//         self.table.add_row(row![headers.join(",")]);
//     }

//     pub fn add_row(&mut self, row: Row) {
//         self.table.add_row(row);
//     }

//     pub fn print_table(&self) {
//         self.table.printstd();
//     }
// }
