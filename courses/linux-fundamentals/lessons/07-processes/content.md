# Processes & Jobs

Every running program is a process. Understanding processes, exit codes, and signals is essential for shell scripting and system administration.

## Exit Codes

Every command returns an exit code (0-255). By convention:
- **0** means success
- **Non-zero** means failure

The special variable `$?` holds the last command's exit code:

```bash
ls /tmp
echo $?        # 0 (success)

ls /nonexistent
echo $?        # 2 (error — no such file)
```

You can set your own exit code with `exit`:

```bash
#!/bin/bash
exit 0         # success
exit 1         # general error
exit 42        # custom error code
```

## Conditional Execution

Use exit codes for control flow:

```bash
command && echo "succeeded"     # runs only if command succeeds
command || echo "failed"        # runs only if command fails

if command; then
    echo "success"
else
    echo "failure"
fi
```

## Signals

Signals are messages sent to processes:

| Signal  | Number | Default Action | Common Use              |
|---------|--------|----------------|------------------------|
| SIGTERM | 15     | Terminate      | Polite shutdown request |
| SIGKILL | 9      | Kill (forced)  | Force kill (can't catch)|
| SIGINT  | 2      | Interrupt      | Ctrl+C                 |
| SIGHUP  | 1      | Hangup         | Terminal closed         |
| SIGUSR1 | 10     | User-defined   | Custom signal           |

Send signals with `kill`:

```bash
kill PID             # sends SIGTERM (default)
kill -9 PID          # sends SIGKILL (force)
kill -USR1 PID       # sends SIGUSR1
```

## Trapping Signals

`trap` lets your script respond to signals:

```bash
trap 'echo "caught signal!"' TERM
trap 'cleanup' EXIT       # runs on any exit
trap '' INT                # ignore SIGINT
```

The EXIT trap runs when the script exits for any reason — great for cleanup.

## The /proc Filesystem

`/proc` is a virtual filesystem exposing kernel and process information:

```bash
cat /proc/cpuinfo         # CPU details
cat /proc/meminfo         # memory stats
cat /proc/$$/status       # current process status
cat /proc/$$/cmdline      # how current process was invoked
```

`$$` is the current process's PID (process ID).

## Background Processes

Run commands in the background with `&`:

```bash
sleep 10 &           # runs in background
echo $!              # PID of last background process
wait                 # wait for all background jobs
wait $PID            # wait for specific process
```

`$!` holds the PID of the most recent background command.

## Subshells

Parentheses create a subshell — a child process with its own environment:

```bash
(cd /tmp && pwd)     # subshell changes dir; parent stays
echo $?              # exit code of the subshell

(exit 42)
echo $?              # 42
```

Variables set in a subshell don't affect the parent:

```bash
X=1
(X=99)
echo $X              # still 1
```

## Key Takeaways

- Exit code 0 = success, non-zero = failure, check with `$?`
- `&&` runs next command only on success, `||` only on failure
- `trap` catches signals — use EXIT trap for cleanup
- `/proc` exposes live system info as files
- `&` backgrounds a process, `wait` pauses until it finishes
- Subshells `()` are isolated — changes don't affect the parent
