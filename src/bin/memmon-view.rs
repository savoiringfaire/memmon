use memmon::{Record, ProcessInfo};
use structopt::StructOpt;
use std::fs::File;
use anyhow::{anyhow, Result};
use std::io::{BufReader, BufRead};
use std::str::FromStr;
use cli_table::{format::Justify, print_stdout, Cell, Style, Table, CellStruct};
use std::cmp;

fn pretty_print_bytes(num: u64) -> String {
    let units = ["B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

    let delimiter: u64 = 1000;

    let exponent = cmp::min(((num as f64).log10().floor() / 3_f64) as u64, units.len() as u64 - 1);

    let final_num = num as f64 / (10_f64.powi(3 * exponent as i32));
    let pretty_bytes = format!("{:.2}", final_num);

    let unit = units[exponent as usize];

    format!("{} {}", pretty_bytes, unit)
}

#[derive(Debug, StructOpt)]
struct Config {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Output file
    #[structopt(short, long, default_value = "process_memory.log")]
    infile: String,

    /// Display the last n records taken
    #[structopt(short, long)]
    tail: Option<i32>,

    /// Display only the top n processes by RSS
    #[structopt(short, long)]
    limit: Option<i32>,
}

struct Parser {
    current_record: Option<Record>,
    records: Vec<Record>,
}

impl Parser {
    fn new() -> Self {
        Self {
            current_record: None,
            records: vec![],
        }
    }

    fn records(self) -> Vec<Record> {
        return self.records;
    }

    fn process_timestamp_line(&mut self, line: String) -> Result<()> {
        // If we're now processing a new record, save the old one
        if let Some(old_record) = self.current_record.clone() {
            self.records.push(old_record);
        }

        self.current_record = Some(Record{
            timestamp: chrono::DateTime::from_str(line.trim_matches(|c| {
                c == '[' || c == ']'
            }))?,
            processes: vec![],
        });

        Ok(())
    }

    fn process_record_line(&mut self, line: String) -> Result<()> {
        // TODO: Move this into a from_str function on Record
        let parts: Vec<&str> = line.split(",").collect();

        if parts.len() != 4 {
            return Err(anyhow!("Bad event line, not enough parts: {}", line));
        }

        let name = parts[0].to_string();
        let pid = parts[1].parse()?;
        let resident_memory = parts[2].parse()?;
        let virtual_memory = parts[3].parse()?;

        if let Some(record) = &mut self.current_record {
            record.add_process(ProcessInfo{
                name,
                pid,
                resident_memory,
                virtual_memory,
            });

            Ok(())
        } else {
            Err(anyhow!("Tried to process record line before timestamp seen"))
        }
    }

    fn process_line(&mut self, line: String) -> Result<()> {
        if line.trim().starts_with("[") {
            self.process_timestamp_line(line)
        } else if line.trim() == "" {
            return Ok(())
        } else if let None = self.current_record {
            Err(anyhow!("Expected first line to be a timestamp"))
        } else {
            self.process_record_line(line)
        }
    }
}

fn main() -> Result<()> {
    let config = Config::from_args();
    let mut parser = Parser::new();

    let file = File::open(&config.infile)?;
    let reader = BufReader::new(file);

    let mut lines = reader.lines();
    while let Some(Ok(line)) = lines.next() {
        parser.process_line(line)?;
    };

    let mut records = parser.records();

    if let Some(count) = config.tail {
        records.truncate(usize::try_from(count)?);
    }

    for record in records {
        println!("\n\n==========\n\n Record Timestamp: {}\n\n", record.timestamp);
        let mut table_vec: Vec<Vec<CellStruct>> = vec![];

        let mut processes = record.processes.clone();

        if let Some(count) = config.limit {
            // This ain't exactly nice -- probably a better way to r-truncate.
            processes.reverse();
            processes.truncate(usize::try_from(count)?);
            processes.reverse();
        }

        for process in processes {
            let mut row: Vec<CellStruct> = vec![];

            row.push(process.name.clone().cell());
            row.push(pretty_print_bytes(process.resident_memory.clone()).cell());
            row.push(pretty_print_bytes(process.virtual_memory.clone()).cell());

            table_vec.push(row);
        }

        let table = table_vec.table()
            .title(vec![
                   "Name",
                   "RSS",
                   "Virt",
            ])
            .bold(true);

        print_stdout(table)?;
    }

    Ok(())
}
