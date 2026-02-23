# Process & Resource Issues

When a server slows to a crawl, the culprit is almost always a process gone wrong.
Understanding how to read process tables, interpret resource usage, and send signals
is the difference between a 5-minute fix and a 5-hour outage.


## Processes and PIDs

Every running program is a **process** with a unique **PID** (Process ID). The kernel
tracks them all. Key commands:

```
ps aux              # snapshot of all processes
ps aux --sort=-%cpu # sorted by CPU usage (highest first)
top                 # live, updating view
htop                # friendlier live view (if installed)
```

A `ps aux` line looks like this:

```
USER       PID %CPU %MEM    VSZ   RSS TTY  STAT START   TIME COMMAND
www-data  1842  0.3  1.2 256000 24576 ?    S    09:15   0:04 nginx: worker
root      2091 98.7  0.1  12000  2048 ?    R    09:30  45:12 runaway_script.sh
```

The columns that matter most during an incident: **PID**, **%CPU**, **%MEM**, **STAT**, and **COMMAND**.


## Signals and kill

Processes respond to **signals**. The `kill` command sends them:

| Signal    | Number | Effect                              |
|-----------|--------|-------------------------------------|
| `SIGTERM` | 15     | Polite shutdown — process can clean up |
| `SIGKILL` | 9      | Forced kill — no cleanup, immediate |
| `SIGHUP`  | 1      | Hangup — often used to reload config |

```
kill 2091           # sends SIGTERM (default)
kill -9 2091        # sends SIGKILL (forced)
kill -HUP 1842      # reload config without restart
```

**Always try SIGTERM first.** SIGKILL is the nuclear option — it leaves no chance for
cleanup, which can cause data corruption or stale lock files.


## Port Conflicts

A port can only be bound by one process at a time. When two services want the same
port, the second one fails to start. Diagnose with:

```
ss -tlnp            # show listening TCP ports with PIDs
netstat -tlnp       # older equivalent
lsof -i :8080       # who's using port 8080?
```

The fix is always one of:
1. **Stop** the conflicting process
2. **Change** one service's port in its config
3. **Kill** the stale process holding the port


## Process States

The `STAT` column in `ps` tells you the process state:

| State | Meaning                                      |
|-------|----------------------------------------------|
| `R`   | Running or runnable                          |
| `S`   | Sleeping (waiting for input/event)           |
| `D`   | Uninterruptible sleep (usually disk I/O)     |
| `Z`   | Zombie — finished but parent hasn't reaped   |
| `T`   | Stopped (suspended)                          |

**Zombies** (`Z`) are dead processes that still have an entry in the process table
because their parent hasn't called `wait()`. They consume no CPU or memory, but too
many of them indicate a buggy parent process.


## Resource Monitoring

During an incident, you need to know what's being exhausted:

- **CPU**: `top`, `ps aux --sort=-%cpu`
- **Memory**: `free -h`, `ps aux --sort=-%mem`
- **Disk**: `df -h` (filesystem usage), `du -sh *` (directory sizes)

```
free -h
              total   used   free  shared  buff/cache  available
Mem:           16G    14G    200M    50M       1.8G       1.5G
```

When "available" is near zero, the OOM killer starts claiming victims.


## Process Trees

Processes have parent-child relationships. Every process has a **PPID** (Parent PID).
You can trace the ancestry:

```
ps -ef              # shows PPID column
pstree              # visual tree
pstree -p           # tree with PIDs
```

This matters during incidents because killing a parent process also kills its children.
Conversely, if a child is misbehaving, you might need to fix or restart the parent.


## Triage Principles

When multiple things are broken simultaneously:

1. **What's causing data loss?** — Fix that first
2. **What's customer-facing?** — Fix that second
3. **What's degraded but functional?** — Fix that third
4. **What's cosmetic?** — Fix that last

Severity isn't about how loud the alert is. It's about **impact** and **urgency**.
