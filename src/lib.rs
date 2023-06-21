use sysinfo::{ProcessExt, PidExt, Process};
use chrono::{Utc, DateTime};

#[derive(Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub resident_memory: u64,
    pub virtual_memory: u64,
    pub name: String,
}

impl From<&Process> for ProcessInfo {
    fn from(process: &Process) -> Self {
        Self {
            name: process.name().to_string(),
            pid: process.pid().as_u32(),
            resident_memory: process.memory(),
            virtual_memory: process.virtual_memory()
        }
    }
}

impl ToString for ProcessInfo {
    fn to_string(&self) -> String {
        format!("{},{},{},{}", self.name, self.pid, self.resident_memory, self.virtual_memory)
    }
}

#[derive(Clone)]
pub struct Record {
    pub timestamp: DateTime<Utc>,
    pub processes: Vec<ProcessInfo>,
}

impl ToString for Record {
    fn to_string(&self) -> String {
        let mut out: String = format!("[{}]\n", self.timestamp);

        let mut sorted_processes = self.processes.clone();
        sorted_processes.sort_by_key(|proc| proc.resident_memory);

        for process in sorted_processes {
            out = format!("{}{}\n", out, process.to_string());
        }

        out
    }
}

impl Record {
    pub fn add_process(&mut self, process: ProcessInfo) {
        self.processes.push(process);
    }
}
