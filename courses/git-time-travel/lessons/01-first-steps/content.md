## Initializing a Repository

Every git project starts with `git init`. This creates a hidden `.git/` directory
that stores all of git's internal data — the object database, refs, config, and
the index (staging area). Without it, you're just a directory with files.

```
git init
```

That's it. One command, and you have version control.


## Staging and Committing

Git uses a two-step process to save work:

1. **Stage** changes with `git add` — this tells git which changes to include
2. **Commit** them with `git commit` — this creates a permanent snapshot

```
git add README.md          # stage one file
git add .                  # stage everything
git commit -m "message"    # save a snapshot
```

A commit is a snapshot of your entire project at one moment in time. Each commit
has a unique hash (like `a1b2c3d`), an author, a timestamp, and a message
describing what changed.


## Checking Status

`git status` is your best friend. It shows you:

- **Staged changes** — ready to be committed (green)
- **Modified files** — changed but not staged (red)
- **Untracked files** — new files git doesn't know about (red)

```
git status
```

Run it constantly. Before staging, after staging, before committing. It always
tells you exactly where things stand.


## Viewing History

`git log` shows the commit history, newest first:

```
git log              # full details
git log --oneline    # compact: one line per commit
```

Each entry shows the commit hash, author, date, and message. The `--oneline`
format is handy for quick scanning — you'll use it a lot in this course.


## Seeing Differences

`git diff` shows what changed in your working directory compared to the last
commit:

```
git diff              # unstaged changes
git diff --staged     # staged changes (what will be committed)
```

This is how you review your work before committing. Always know what you're
about to save.


## Selective Staging

You don't have to stage everything. `git add` takes specific filenames:

```
git add file1.txt file3.txt    # stage only these two
git commit -m "update docs"
```

This lets you make focused commits — each one about a single logical change.
The files you didn't stage remain modified in your working directory.


## Amending Commits

Made a typo in your last commit message? Forgot to stage a file?

```
git commit --amend -m "corrected message"
```

`--amend` replaces the most recent commit. It's safe as long as you haven't
pushed the commit to a shared repository. Think of it as an eraser for your
last mistake.
