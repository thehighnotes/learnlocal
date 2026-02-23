## The Alert Fires

3:47 AM. Your phone buzzes. The monitoring system says something is down.
What do you do first?

**Not panic.** That is always step one. The second step is to gather
information before touching anything. Most incidents get worse when
someone "fixes" things before understanding the problem.


## Read Before You Fix

Your first instinct will be to restart the service. Resist it. A restart
might destroy the evidence you need. Before you change anything:

1. **Read the logs** — they tell you what happened and when
2. **Check what's running** — is the process alive? Is it stuck?
3. **Check disk space** — full disks cause bizarre failures
4. **Look at recent changes** — configs, deploys, cron jobs

```bash
ls app/logs/               # what log files exist?
cat app/logs/error.log     # read the error log
grep "ERROR" app/logs/*    # search all logs for errors
```


## Quick Diagnosis Commands

These five commands answer most first-response questions:

| Command         | What it tells you                     |
|-----------------|---------------------------------------|
| `ls`            | What files exist, when they changed   |
| `cat` / `less`  | File contents                         |
| `grep`          | Find specific text in files           |
| `ps aux`        | What processes are running            |
| `df -h`         | Disk space usage                      |

You will use all of these in the exercises that follow.


## grep: Your Best Friend

`grep` searches for patterns in files. During an incident, you will
grep constantly:

```bash
grep "ERROR" logfile.txt           # find error lines
grep -i "timeout" logfile.txt      # case-insensitive search
grep -n "CRITICAL" logfile.txt     # show line numbers
grep -r "ERROR" app/logs/          # search recursively
```

The `-n` flag is especially useful — it tells you exactly where in
the file the problem is.


## Comparing Files

When a config change breaks things, you need to spot what changed.
`diff` compares two files line by line:

```bash
diff old.conf new.conf             # show differences
diff -u old.conf new.conf          # unified format (easier to read)
```

Lines starting with `-` were removed, lines with `+` were added.


## Writing Status Updates

During an incident, communication matters as much as technical work.
A good status update has three parts:

1. **What happened** — the symptom, not your guess at the cause
2. **When** — timestamps from the logs
3. **Current state** — what you know, what you are doing next

Keep it short. People are waiting.
