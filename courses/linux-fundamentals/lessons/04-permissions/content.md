# Permissions & Ownership

Linux is a multi-user system. Permissions control who can read, write, and execute each file.

## The Permission Model

Every file has three sets of permissions for three classes of users:

| Class   | Symbol | Who                         |
|---------|--------|-----------------------------|
| User    | `u`    | The file's owner            |
| Group   | `g`    | Members of the file's group |
| Other   | `o`    | Everyone else               |

Each class has three permission bits:

| Permission | Symbol | Octal | Effect on files    | Effect on dirs        |
|------------|--------|-------|--------------------|----------------------|
| Read       | `r`    | 4     | View contents      | List contents         |
| Write      | `w`    | 2     | Modify contents    | Create/delete files   |
| Execute    | `x`    | 1     | Run as program     | Enter directory (cd)  |

## Reading Permission Strings

`ls -l` shows permissions as a 10-character string:

```
-rwxr-xr--  1 alice staff  4096 Jan 15 09:30 script.sh
```

Breaking it down: `-` (file type) `rwx` (user) `r-x` (group) `r--` (other)

- User (alice): read + write + execute
- Group (staff): read + execute
- Other: read only

## Octal Notation

Each permission set is a sum: r=4, w=2, x=1

```
rwx = 4+2+1 = 7
r-x = 4+0+1 = 5
r-- = 4+0+0 = 4
```

So `rwxr-xr--` = **754**

Common patterns:

| Octal | Meaning              | Typical use            |
|-------|----------------------|------------------------|
| 755   | rwxr-xr-x            | Executable scripts     |
| 644   | rw-r--r--            | Regular files          |
| 700   | rwx------            | Private executables    |
| 600   | rw-------            | Private files (SSH keys)|

## chmod: Changing Permissions

**Octal mode:**

```bash
chmod 755 script.sh      # rwxr-xr-x
chmod 644 readme.txt     # rw-r--r--
```

**Symbolic mode:**

```bash
chmod u+x script.sh      # add execute for user
chmod g-w file.txt        # remove write for group
chmod o=r file.txt        # set other to read-only
chmod a+r file.txt        # add read for all (user+group+other)
```

## Reading Permissions with stat

`stat` gives you the octal permissions directly:

```bash
stat --format='%a' file.txt     # prints "644"
stat --format='%A' file.txt     # prints "-rw-r--r--"
```

## umask: Default Permissions

`umask` determines what permissions new files and directories get.
It works by subtracting from the maximum (666 for files, 777 for dirs):

```bash
umask          # show current mask (e.g., 0022)
umask 0077     # set very restrictive defaults
```

With umask 0022:
- New files: 666 - 022 = 644 (rw-r--r--)
- New directories: 777 - 022 = 755 (rwxr-xr-x)

## Directory Permissions

Directories need special attention:
- **Read (r):** You can list contents with `ls`
- **Execute (x):** You can `cd` into it and access files inside
- **Write (w):** You can create and delete files inside

Without `x`, you can't even access files inside the directory, even with `r`.

## Key Takeaways

- Every file has rwx permissions for user, group, and other
- Octal notation sums r=4, w=2, x=1 for each class
- `chmod` changes permissions, `stat --format='%a'` reads them
- `umask` controls defaults for new files
- Directory `x` permission is needed to enter or access files within
