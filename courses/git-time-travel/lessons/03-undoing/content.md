# Undoing Things

Git's real superpower isn't tracking changes — it's letting you undo them.
Whether you staged the wrong file, committed too early, or need to reverse
a published change, git has a tool for every situation.

## The Three Trees

Git manages your work in three layers:

| Layer              | What lives there                        | Key command       |
|--------------------|-----------------------------------------|-------------------|
| **Working Directory** | Your actual files on disk              | `git restore`     |
| **Staging Area**      | Changes queued for the next commit     | `git restore --staged` |
| **HEAD (Repository)** | The last committed snapshot            | `git reset`, `git revert` |

Every undo operation moves changes between these layers — or removes them entirely.

## Unstaging: Moving Changes Back from Staging

You ran `git add` but changed your mind. No problem:

```bash
git restore --staged file.txt    # unstage file.txt (changes stay in working dir)
```

The older equivalent is `git reset HEAD file.txt`, but `git restore --staged`
is the modern, clearer way.

## Discarding Working Directory Changes

You edited a file but want to throw away the changes:

```bash
git restore file.txt             # discard changes, restore to last commit
```

**Warning:** This is destructive. Uncommitted changes are gone forever.

## git reset — Moving HEAD

`git reset` moves the HEAD pointer backward, effectively "uncommitting" work.
It has three modes that differ in what happens to your changes:

| Mode      | HEAD moves? | Staging cleared? | Working dir cleared? |
|-----------|:-----------:|:----------------:|:--------------------:|
| `--soft`  | Yes         | No               | No                   |
| `--mixed` | Yes         | Yes              | No                   |
| `--hard`  | Yes         | Yes              | Yes                  |

```bash
git reset --soft HEAD~1    # undo last commit, keep changes staged
git reset HEAD~1           # undo last commit, unstage changes (--mixed is default)
git reset --hard HEAD~1    # undo last commit, discard all changes (dangerous!)
```

`HEAD~1` means "one commit before HEAD". `HEAD~2` means two back, etc.

### When to use each mode

- **--soft**: You committed too early and want to add more changes to that commit.
- **--mixed** (default): You committed and want to rethink what to stage.
- **--hard**: You want to completely throw away recent work. Use with care.

## git revert — Safe Undo for Published History

`git reset` rewrites history — fine for local commits, but dangerous if
you've already pushed. `git revert` is the safe alternative:

```bash
git revert <commit-hash>   # create a NEW commit that undoes the target commit
```

Revert doesn't delete the bad commit. It adds a new commit that applies
the inverse of the changes. History is preserved, collaborators aren't confused.

```
Before:  A → B → C (HEAD)
Revert B: A → B → C → B' (HEAD)    # B' undoes B's changes
```

## git clean — Removing Untracked Files

Untracked files (not in git at all) won't be touched by `restore` or `reset`.
Use `git clean` to remove them:

```bash
git clean -n               # dry run — show what would be deleted
git clean -f               # actually delete untracked files
git clean -fd              # delete untracked files AND directories
```

The `-f` flag is required — git won't clean without it (safety measure).

## Choosing the Right Undo Tool

| Situation                           | Command                        |
|-------------------------------------|--------------------------------|
| Unstage a file                      | `git restore --staged file`    |
| Discard working directory changes   | `git restore file`             |
| Undo last commit, keep staged       | `git reset --soft HEAD~1`      |
| Undo last commit, keep in work dir  | `git reset HEAD~1`             |
| Undo last commit, discard all       | `git reset --hard HEAD~1`      |
| Safely undo a published commit      | `git revert <hash>`            |
| Remove untracked files              | `git clean -f`                 |

## Key Takeaways

- `git restore` handles working directory and staging area changes
- `git reset` moves HEAD backward with three strictness levels
- `git revert` is the only safe undo for shared/published commits
- `git clean -f` removes files git doesn't track
- When in doubt, use `--soft` or `revert` — they're the least destructive
