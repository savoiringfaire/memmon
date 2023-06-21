use sysinfo::{ProcessExt, PidExt, Process, Pid};
use chrono::{Utc, DateTime};

#[derive(Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub cmd: String,
    pub parent: u32,
    pub start_time: u64,
    pub resident_memory: u64,
    pub virtual_memory: u64,
}

impl From<&Process> for ProcessInfo {
    fn from(process: &Process) -> Self {
        Self {
            name: process.name().to_string(),
            pid: process.pid().as_u32(),
            cmd: process.cmd().join(" ").to_string().replace(",", "_").replace("\n", ""),
            parent: process.parent().unwrap_or(Pid::from(0)).as_u32(),
            start_time: process.start_time(),
            resident_memory: process.memory(),
            virtual_memory: process.virtual_memory()
        }
    }
}

impl ToString for ProcessInfo {
    fn to_string(&self) -> String {
        format!("{},{},{},{},{},{},{}", self.name, self.cmd, self.parent, self.start_time, self.pid, self.resident_memory, self.virtual_memory)
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
