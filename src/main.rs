mod cgroups;
use self::cgroups::CpusetCgroup;
use clap::Parser;
use std::error::Error;
use std::process::Command;

#[derive(Parser)]
#[command(author, version, about = "Run commands in an isolated CPU environment")]
struct Cli {
    /// Command to execute
    #[arg(required = true)]
    command: String,

    /// Arguments for the command
    #[arg(trailing_var_arg = true)]
    args: Vec<String>,

    /// CPU cores to use (comma-separated list)
    #[arg(short, long, value_delimiter = ',', default_value = "0,1")]
    cpus: Vec<u32>,

    /// Name of the cgroup
    #[arg(short, long, default_value = "cleanroom")]
    name: String,
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = main_inner(&cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn main_inner(cli: &Cli) -> Result<(), Box<dyn Error>> {
    let cgroup = CpusetCgroup::create(&cli.name)?;
    cgroup.set_cpu_exclusive(true)?;
    cgroup.set_cpus(&cli.cpus)?;

    // Spawn the process
    let child = Command::new(&cli.command).args(&cli.args).spawn()?;

    // Add the spawned process to cgroup
    cgroup.add_process(child.id() as u32)?;

    // Wait for the child process to complete
    let status = child.wait_with_output()?;

    // Clean up the cgroup
    cgroup.delete()?;

    // Exit with the same status as the child process
    if !status.status.success() {
        std::process::exit(status.status.code().unwrap_or(1));
    }

    Ok(())
}
