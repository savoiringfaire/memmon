use chrono::{Utc, Duration};
use anyhow::{Result, Error};
use sysinfo::{System, SystemExt};
use std::collections::VecDeque;
use std::thread;
use std::convert::From;
use std::io::prelude::*;
use structopt::StructOpt;
use std::str::FromStr;
use std::ops::Deref;
use parse_duration::parse;

use memmon::{ Record, ProcessInfo };

#[derive(Debug)]
struct ConfigDuration(Duration);

impl Deref for ConfigDuration {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Needed for Config to work below
impl FromStr for ConfigDuration {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, <Self as FromStr>::Err> {
        Ok(ConfigDuration(Duration::from_std(parse(input)?)?))
    }
}

#[derive(Debug, StructOpt)]
struct Config {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Set the period of measurements
    #[structopt(short, long, default_value = "60s")]
    period: ConfigDuration,

    /// How long to keep history for
    #[structopt(short, long, default_value = "2h")]
    history: ConfigDuration,

    /// Output file
    #[structopt(short, long, default_value = "process_memory.log")]
    outfile: String,
}


struct Monitor {
    records: VecDeque<Record>,
    sys: System,
    config: Config
}

impl ToString for Monitor {
    fn to_string(&self) -> String {
        let mut out: String = "".to_string();

        for record in &self.records {
            out = format!("{}{}\n", out, record.to_string())
        }

        out
    }
}

impl Monitor {
    fn new(config: Config) -> Monitor {
        let needed_entries = config.history.num_seconds() / config.period.num_seconds();

        Monitor {
            records: VecDeque::with_capacity(needed_entries.try_into().unwrap()),
            sys: System::new_all(),
            config
        }
    }

    fn insert_record(&mut self, record: Record) -> Result<()> {
        if self.records.len() == self.records.capacity() {
            self.records.pop_front();
        }

        self.records.push_back(record);

        Ok(())
    }

    fn record(&mut self) -> Result<()> {
        if self.config.debug {
            println!("Recording new value")
        }

        // TODO: No need to refresh everything here.
        self.sys.refresh_all();

        let mut processes: Vec<ProcessInfo> = Vec::with_capacity(self.sys.processes().len());

        for (_, proc) in self.sys.processes() {
            processes.push(ProcessInfo::from(proc));
        }

        self.insert_record(Record{
            timestamp: Utc::now(),
            processes,
        })
    }

    fn save(&mut self) -> Result<()> {
        if self.config.debug {
            println!("Persisting current records to file")
        }

        let mut file = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(&self.config.outfile)?;

        file.write_all(self.to_string().as_bytes())?;

        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        loop {
            match self.record() {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("Error recording values: {}", error);
                }
            }

            match self.save() {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("Error saving values to file: {}", error);
                }
            }

            thread::sleep(self.config.period.to_std()?);
        }
    }
}

fn main() -> Result<()> {
    let config = Config::from_args();

    let mut monitor = Monitor::new(config);

    monitor.run()?;

    Ok(())
}
