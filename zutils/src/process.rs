use anyhow::{anyhow, Result};
use sysinfo::{Pid, Process, Signal, System};

#[derive(Debug)]
pub struct SystemProcess {
    system: System,
}

impl Default for SystemProcess {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemProcess {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self { system }
    }

    pub fn find_process(&mut self, command_substring: &str) -> Vec<(&Pid, &Process)> {
        self.system.refresh_all();
        self.system
            .processes()
            .iter()
            .filter(|(_, process)| {
                process
                    .cmd()
                    .join(" ")
                    .contains(&command_substring.to_string())
            })
            .collect()
    }

    pub fn kill_process(&mut self, command_substring: &str) -> Result<()> {
        let processes: Vec<(&Pid, &Process)> = self.find_process(command_substring);

        for (pid, process) in processes {
            if process.kill_with(Signal::Kill).is_none() {
                return Err(anyhow!("Error while killing the process with PID {pid}"));
            }
            println!(
                "Killed '{}' process with PID: {}",
                process.cmd().join(" "),
                pid
            );
        }

        Ok(())
    }
}
