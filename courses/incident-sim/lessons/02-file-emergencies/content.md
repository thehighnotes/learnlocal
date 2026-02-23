## When the Filesystem Fights Back

The application is down. Logs say "Permission denied" or "No such file."
These are filesystem problems, and they are among the most common
causes of production incidents.


## Linux Permissions in 60 Seconds

Every file has three permission sets: user (owner), group, and other.
Each set has three bits: read (r=4), write (w=2), execute (x=1).

```
-rw-r--r--  = 644 = owner reads/writes, everyone else reads
-rwxr-xr-x  = 755 = owner does everything, everyone else reads/executes
----------  = 000 = nobody can do anything (broken)
```

Check permissions:
```bash
ls -l file.txt          # shows -rw-r--r--
stat --format='%a' file.txt   # shows 644
```

Fix permissions:
```bash
chmod 644 file.txt      # set to rw-r--r--
chmod 755 script.sh     # set to rwxr-xr-x
chmod u+r file.txt      # add read for owner
```


## Copying and Restoring Files

When a config file gets deleted or corrupted, you restore from backup:

```bash
cp backup/server.conf app/config/server.conf    # copy backup into place
cp -r backup/configs/ app/configs/               # copy entire directory
```

The `-r` flag copies directories recursively. Without it, `cp` only
handles individual files.


## Symlinks: Pointers to Files

A symbolic link is a pointer to another file or directory. Deployments
often use symlinks to switch between versions:

```
current -> v2.0/       # "current" points to the v2.0 directory
```

Create and fix symlinks:
```bash
ln -s target link_name          # create a symlink
ln -sf new_target link_name     # force-replace an existing symlink
readlink link_name              # show where a symlink points
```

A broken symlink points to something that doesn't exist. `ls -l` will
show it, and the target will be highlighted in red on most terminals.


## Directory Structure

Applications expect certain directories to exist. If they don't, the
app crashes on startup with unhelpful error messages:

```bash
mkdir -p /data/cache/sessions     # create nested directories
```

The `-p` flag creates parent directories as needed. Without it, `mkdir`
fails if the parent doesn't exist.


## Emergency Backups

Before making a risky fix, back up what you have:

```bash
mkdir backup
cp app/*.conf backup/           # copy all config files
cp -r app/data backup/data      # copy data directory
```

A bad fix on top of a broken system is worse than the original problem.
Back up first, then fix.


## Key Principles

1. **Check permissions first** — "Permission denied" is the #1 red herring
2. **Restore from backup** — faster than recreating from scratch
3. **Fix symlinks** — broken links cause "No such file" even when the file exists
4. **Create missing directories** — apps don't create their own
5. **Back up before fixing** — so you can undo your fix if it makes things worse
