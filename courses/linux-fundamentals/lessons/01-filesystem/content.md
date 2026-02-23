# Filesystem & Navigation

Everything in Linux is organized as a tree of files and directories rooted at `/`.
Understanding this structure is the first step to working effectively in the terminal.

## The Filesystem Hierarchy

Linux follows a standard layout called the **Filesystem Hierarchy Standard (FHS)**:

| Path       | Purpose                                    |
|------------|--------------------------------------------|
| `/`        | Root — the top of the entire tree          |
| `/home`    | User home directories (`/home/alice`)      |
| `/etc`     | System configuration files                 |
| `/usr`     | User programs, libraries, documentation    |
| `/tmp`     | Temporary files (cleared on reboot)        |
| `/var`     | Variable data — logs, caches, mail         |
| `/proc`    | Virtual filesystem — live kernel/process info |
| `/bin`     | Essential command binaries (ls, cp, cat)   |
| `/dev`     | Device files (disks, terminals, etc.)      |

Your personal files live in `/home/<username>`. The shorthand `~` refers to your home directory.

## Navigating with `pwd`, `cd`, and `ls`

Three commands you'll use constantly:

- **`pwd`** — Print Working Directory. Shows where you are right now.
- **`cd <path>`** — Change Directory. Moves you to a new location.
- **`ls`** — List. Shows files and directories in the current location.

```bash
pwd          # /home/alice
cd /tmp      # move to /tmp
ls           # list what's in /tmp
cd ~         # back to home
```

## Absolute vs Relative Paths

An **absolute path** starts from root (`/`): `/home/alice/documents/report.txt`

A **relative path** starts from your current directory: `documents/report.txt`

Special directory names:
- `.` — the current directory
- `..` — the parent directory (one level up)

```bash
cd /home/alice
cd ..          # now at /home
cd ./alice     # back to /home/alice (same as cd alice)
```

## Hidden Files

Files whose names start with `.` are hidden from normal `ls` output.
Use `ls -a` to show them:

```bash
ls           # shows: Documents  Downloads  Music
ls -a        # shows: .  ..  .bashrc  .config  Documents  Downloads  Music
```

Common hidden files: `.bashrc` (shell config), `.ssh/` (SSH keys), `.config/` (app settings).

## Useful `ls` Flags

| Flag  | Effect                                |
|-------|---------------------------------------|
| `-a`  | Show all files including hidden       |
| `-l`  | Long format (permissions, size, date) |
| `-h`  | Human-readable sizes (with `-l`)      |
| `-R`  | Recursive — list subdirectories too   |
| `-1`  | One file per line                     |

## Key Takeaways

- Everything lives under `/` in a tree structure
- `pwd` shows where you are, `cd` moves you, `ls` lists contents
- Absolute paths start with `/`, relative paths don't
- `.` is here, `..` is parent, `~` is home
- Files starting with `.` are hidden — use `ls -a` to see them
