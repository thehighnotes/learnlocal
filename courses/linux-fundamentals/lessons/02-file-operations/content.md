# File Operations

Now that you can navigate the filesystem, let's learn to create, copy, move, and remove files and directories.

## Creating Files and Directories

**`touch`** creates empty files (or updates timestamps on existing ones):

```bash
touch notes.txt           # creates notes.txt
touch a.txt b.txt c.txt   # creates multiple files
```

**`mkdir`** creates directories. Use `-p` to create parent directories:

```bash
mkdir photos              # single directory
mkdir -p a/b/c/d          # creates entire path
```

## Copying with `cp`

```bash
cp source.txt dest.txt          # copy a file
cp -r source_dir/ dest_dir/     # copy a directory (recursive)
cp file1.txt file2.txt dest/    # copy multiple files into a directory
```

The `-r` (recursive) flag is required for directories — without it, `cp` refuses to copy directories.

## Moving and Renaming with `mv`

`mv` both moves and renames — there's no separate rename command:

```bash
mv old_name.txt new_name.txt    # rename
mv file.txt /tmp/               # move to /tmp
mv dir1/ /tmp/dir2/             # move+rename directory
```

## Removing with `rm`

```bash
rm file.txt              # remove a file
rm -r directory/          # remove directory and contents
rm -f locked_file.txt    # force remove (no confirmation)
rm -rf old_project/      # remove directory, force, no prompt
```

Be very careful with `rm -rf` — there is no undo.

## Links: Hard and Soft

A **hard link** is another name for the same file data (same inode):

```bash
ln original.txt hardlink.txt     # hard link
```

Both names point to the same data on disk. Deleting one doesn't affect the other. Hard links can't cross filesystems and can't link to directories.

A **soft link** (symlink) is a pointer to a path:

```bash
ln -s /path/to/target linkname   # symbolic link
```

Symlinks can cross filesystems, link to directories, and break if the target is deleted.

## Checking File Types

Use `test` or `[` to check what something is:

```bash
test -f notes.txt && echo "regular file"
test -d photos/ && echo "directory"
test -L linkname && echo "symbolic link"
```

The `stat` command shows detailed file information including inode number:

```bash
stat --format='%i' file.txt      # print inode number
```

## Key Takeaways

- `touch` creates files, `mkdir -p` creates directory trees
- `cp -r` for directories, `mv` for move/rename, `rm -r` for directory removal
- Hard links share inodes; symlinks are flexible but can break
- `test -f/-d/-L` checks file type, `stat` shows metadata
