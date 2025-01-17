import subprocess
import time
import argparse
import os
from contextlib import contextmanager

def print_cgroups():
    cgroup_dir = "/sys/fs/cgroup"
    if os.path.exists(cgroup_dir):
        print("printing cgroups...")
        for root, dirs, files in os.walk(cgroup_dir):
            for name in dirs:
                print(os.path.join(root, name))
    else:
        print("Cpuset cgroup directory does not exist.")

def print_processes():
    try:
        result = subprocess.run(['ps', '-e', '-o', 'pid,comm'], check=True, capture_output=True, text=True)
        print("printing processes...")
        print(result.stdout)
    except subprocess.CalledProcessError as e:
        print(f"Error: {e.stderr}")

class CpusetCgroupV2:
    def __init__(self, name: str, cpus: str, cgroup_root="/sys/fs/cgroup"):
        self.name = name
        self.cpus = cpus
        self.dir = f"{cgroup_root}/{self.name}"

    def create(self):
        try:
            print(f"Creating cpuset {self.name} with '{self.cpus}' CPUs")
            os.makedirs(self.dir, exist_ok=True)
            # make the cpuset partition a root, so that it can get exclusive access to the CPUs
            with open(os.path.join(self.dir, "cpuset.cpus.partition"), "w") as f:
                f.write("root")
            # set the cpus that you want to make exclusive
            with open(os.path.join(self.dir, "cpuset.cpus.exclusive"), "w") as f:
                f.write(self.cpus)
            print(f"Created cpuset {self.name} with '{self.cpus}' CPUs")
        except Exception as e:
            print(f"Error creating cpuset: {e}")
            raise

    def delete(self):
        try:
            if os.path.exists(self.dir):
                os.rmdir(self.dir)
                print(f"Deleted cpuset {self.name}")
            else:
                print(f"Cpuset {self.name} does not exist.")
        except Exception as e:
            print(f"Error deleting cpuset: {e}")

    def add_process(self, pid: int):
        try:
            tasks_file = os.path.join(self.dir, "cgroup.procs")
            with open(tasks_file, "w") as f:
                f.write(str(pid))
            print(f"Added process {pid} to cpuset {self.name}")
        except Exception as e:
            print(f"Error adding process to cpuset: {e}")

@contextmanager
def cpuset_context(cpuset_name: str, cpus: str):
    cpuset = CpusetCgroupV2(cpuset_name, cpus)
    try:
        cpuset.create()
        yield cpuset
    finally:
        cpuset.delete()

def validate_cpus(cpus):
    if not cpus:
        raise argparse.ArgumentTypeError("CPUs cannot be empty")
    if not all(part.isdigit() or (part.count('-') == 1 and all(p.isdigit() for p in part.split('-'))) for part in cpus.split(',')):
        raise argparse.ArgumentTypeError("Invalid CPU format. Expected format: '1-2,4,6-8'")
    return cpus

def main():
    parser = argparse.ArgumentParser(description="Execute a command with arguments.")
    parser.add_argument('--cpuset', help="Name of the cpuset to create", default="cleanroom")
    parser.add_argument('--cpus', help="Number of CPUs for the cpuset", default="1-2", type=validate_cpus)
    parser.add_argument('--cgroup-root', help="Root of the cgroup", default="/sys/fs/cgroup", type=str)
    parser.add_argument('executable', help="The executable to run")
    parser.add_argument('args', nargs=argparse.REMAINDER, help="Arguments for the executable")

    args = parser.parse_args()
    print(args)

    with cpuset_context(args.cpuset, args.cpus, args.cgroup_root) as cgroup:
        # TODO: sleep for now to let processes be descheduled. Check if this
        # wait is even necessary.
        print("sleeping to let processes be descheduled off of target CPUs...")
        time.sleep(0.5)
        try:
            process = subprocess.Popen(
                [args.executable] + args.args,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )
            cgroup.add_process(process.pid)
            stdout, stderr = process.communicate()
            if process.returncode != 0:
                raise subprocess.CalledProcessError(
                    process.returncode,
                    [args.executable] + args.args,
                    stdout,
                    stderr
                )
            print("wrapped!")
            print(stdout)
        except subprocess.CalledProcessError as e:
            print(f"Error: {e.stderr}")

if __name__ == "__main__":
    main()