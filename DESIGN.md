# Cleanroom Design

Cleanroom strives for resource isolation for the target process to ensure that
the target executable can have consistent performance (e.g. for benchmarking).
The following is employed:

1. Exclusive usage of CPUs, isolated from system threads.

## CPU pinning and isolation

The performance of a process can be affected by:

1. Cache locality: if a thread runs on the same CPU, it can leverage the L1/L2
   cache of a processor. Inconsistent cache coherence (for example, a separate
   thread being scheduled on a CPU invalidating the cache) can result in
   different latency than a cache hit.
2. Inconsistent scheduling: If a CPU that was originally running the benchmark
   has a system thread executed on it, the cost of a context switch may result
   in increased latency.

This will be solved by using [control
groups](https://docs.kernel.org/admin-guide/cgroup-v2.html) in the Linux kernel,
ensuring both "pinning" of the target process on the desired CPUs, as well as
isolation, ensuring that no other thread runs in the CPU.

### Isolating a process via cgroups v2

For Cgroups V2, this can look like the following:

```bash
CGROUP_DIR=/sys/fs/cgroup/cleanroom
mkdir -p ${CGROUP_DIR}
# set the cpuset partition as root, enabling it
# to use cpus exclusively.
echo "root" > "${CGROUP_DIR}/cpuset.cpus.partition"
# set the CPUS you want to give to your cpuset exclusively
echo "root" > "${CGROUP_DIR}/cpuset.cpus.exclusive"
# verify that you have exclusive access
cat "${CGROUP_DIR}/cpuset.cpus.exclusive.effective"
```

Specifically:

- `cpuset.cpus.partition` can be used to create a separate root cpuset which can
  ensure exclusive access to those CPUs.
- `cpuset.cpus.exclusive` is set to the target CPUs for which exclusive access
  should be given (e.g. 0-1 for the first two CPUs).

### Consider hyperthreading for CPU assignment

Some CPUs detected by the Linux kernel are "logical", in that they do not have a
physical presence on the host. These are typically
[hyperthreaded](https://en.wikipedia.org/wiki/Hyper-threading), where a single
core can execute the same instruction on two different sets of data.

When choosing your CPUs, either select CPUs that are not hyperthreaded, or
select a CPU and all of it's hyperthreaded logical cores.

You can tell by seeing which "core" the CPU is attached to. In the example below,
CPU 0/1 are the physical and hyperthreaded CPU for core 0:

```bash
lscpu -e
CPU NODE SOCKET CORE L1d:L1i:L2:L3 ONLINE    MAXMHZ   MINMHZ      MHZ
  0    0      0    0 0:0:0:0          yes 4800.0000 400.0000 1299.986
  1    0      0    0 0:0:0:0          yes 4800.0000 400.0000 1300.000
  2    0      0    1 4:4:1:0          yes 4800.0000 400.0000 1292.786
  3    0      0    1 4:4:1:0          yes 4800.0000 400.0000  625.185
  4    0      0    2 8:8:2:0          yes 3600.0000 400.0000 1299.984
  5    0      0    3 9:9:2:0          yes 3600.0000 400.0000 1299.997
  6    0      0    4 10:10:2:0        yes 3600.0000 400.0000 1299.960
  7    0      0    5 11:11:2:0        yes 3600.0000 400.0000 1299.961
  8    0      0    6 12:12:3:0        yes 3600.0000 400.0000 1299.844
  9    0      0    7 13:13:3:0        yes 3600.0000 400.0000 1300.107
 10    0      0    8 14:14:3:0        yes 3600.0000 400.0000 1154.010
 11    0      0    9 15:15:3:0        yes 3600.0000 400.0000 1299.940
```