# Users, Networking & Services

This final lesson covers user identity, system information, environment variables, and networking basics — tools you'll use daily for system administration.

## User Identity

**`whoami`** — prints your username:

```bash
whoami              # alice
```

**`id`** — shows user ID, group ID, and group memberships:

```bash
id                  # uid=1000(alice) gid=1000(alice) groups=1000(alice),27(sudo)
id -u               # just the numeric user ID
id -g               # just the numeric group ID
id -Gn              # list group names
```

## The /etc/passwd File

`/etc/passwd` stores user account information, one line per user:

```
root:x:0:0:root:/root:/bin/bash
alice:x:1000:1000:Alice Smith:/home/alice:/bin/bash
nobody:x:65534:65534:nobody:/nonexistent:/usr/sbin/nologin
```

Fields (colon-separated):
1. Username
2. Password placeholder (x = stored in /etc/shadow)
3. User ID (UID)
4. Group ID (GID)
5. Comment (full name)
6. Home directory
7. Login shell

Parse with `cut` or `awk`:

```bash
cut -d':' -f1 /etc/passwd         # list all usernames
awk -F':' '$3 >= 1000' /etc/passwd # users with UID >= 1000
```

## Environment Variables

Environment variables configure your shell and programs:

```bash
echo $HOME          # your home directory
echo $USER          # your username
echo $PATH          # executable search path
echo $SHELL         # your default shell
echo $LANG          # language/locale setting

env                 # list ALL environment variables
printenv HOME       # print specific variable
```

Set variables:

```bash
MY_VAR="hello"      # shell variable (not inherited)
export MY_VAR       # environment variable (inherited by child processes)
export DB_HOST=localhost  # set and export in one step
```

## System Information

```bash
hostname               # machine name
uname -a               # kernel version and architecture
uname -r               # just the kernel release
uname -m               # architecture (x86_64, aarch64)
uptime                 # how long the system has been running
```

## Networking Basics

```bash
hostname -I            # IP addresses
ip addr show           # detailed network interfaces
ip route show          # routing table
ss -tlnp               # listening TCP ports
```

`/etc/hostname` contains the machine name. `/etc/hosts` maps hostnames to IPs.

## Service Management (systemd)

Most modern Linux systems use `systemd` to manage services:

```bash
systemctl status nginx       # check service status
systemctl start nginx        # start a service
systemctl stop nginx         # stop a service
systemctl enable nginx       # start on boot
systemctl list-units         # list active services
```

Note: Service management requires elevated privileges (sudo) and isn't covered in exercises since we run in a sandbox.

## Key Takeaways

- `whoami` and `id` reveal user identity and group membership
- `/etc/passwd` stores user accounts in a parseable colon-delimited format
- Environment variables like `$HOME`, `$USER`, `$PATH` configure your environment
- `hostname`, `uname` provide system information
- `ip addr`, `ss` show networking state
- `systemctl` manages systemd services
