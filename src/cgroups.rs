use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

pub struct CpusetCgroup {
    path: PathBuf,
}

impl CpusetCgroup {
    /// Creates a new cpuset cgroup with the given name
    pub fn create(name: &str) -> Result<Self, std::io::Error> {
        let cgroup_path = PathBuf::from("/sys/fs/cgroup/cpuset").join(name);

        // Create the cgroup directory if it doesn't exist
        fs::create_dir_all(&cgroup_path)?;

        Ok(CpusetCgroup { path: cgroup_path })
    }

    pub fn add_process(&self, pid: u32) -> Result<(), std::io::Error> {
        let cgroup_path = self.path.join("cgroup.procs");
        fs::write(cgroup_path, pid.to_string())
    }

    pub fn delete(&self) -> Result<(), std::io::Error> {
        fs::remove_dir_all(&self.path)
    }

    /// Sets CPU exclusivity for this cpuset cgroup
    pub fn set_cpu_exclusive(&self, exclusive: bool) -> Result<(), std::io::Error> {
        let cpu_exclusive_path = self.path.join("cpuset.cpu_exclusive");
        fs::write(cpu_exclusive_path, if exclusive { "1" } else { "0" })
    }

    /// Assigns the specified CPUs to this cpuset cgroup
    pub fn set_cpus(&self, cpus: &[u32]) -> Result<(), std::io::Error> {
        if cpus.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "CPU list cannot be empty",
            ));
        }

        // Convert CPU list to the format expected by cgroups (e.g., "0-2,4,6-8")
        let cpu_list = format_cpu_list(cpus);
        let cpuset_path = self.path.join("cpuset.cpus");
        fs::write(cpuset_path, cpu_list)
    }
}

/// Helper function to format CPU list in cgroup-compatible format
fn format_cpu_list(cpus: &[u32]) -> String {
    let mut cpus = cpus.to_vec();
    cpus.sort_unstable();

    let mut ranges = Vec::new();
    let mut range_start = cpus[0];
    let mut prev = cpus[0];

    for &cpu in cpus.iter().skip(1).chain(std::iter::once(&(prev + 2))) {
        if cpu > prev + 1 {
            if range_start == prev {
                ranges.push(range_start.to_string());
            } else {
                ranges.push(format!("{}-{}", range_start, prev));
            }
            range_start = cpu;
        }
        prev = cpu;
    }

    ranges.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_cpu_list() {
        assert_eq!(format_cpu_list(&[0, 1, 2, 4, 6, 7, 8]), "0-2,4,6-8");
        assert_eq!(format_cpu_list(&[0]), "0");
        assert_eq!(format_cpu_list(&[0, 1]), "0-1");
        assert_eq!(format_cpu_list(&[0, 2, 4]), "0,2,4");
    }
}
