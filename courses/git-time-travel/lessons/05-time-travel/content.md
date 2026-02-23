## The Reflog: Git's Secret Memory

Git has a hidden log that tracks every move your HEAD makes. It's called the
**reflog** (reference log), and it remembers things even after they vanish from
`git log`.

```
git reflog
```

Every checkout, commit, reset, merge, rebase, and cherry-pick gets recorded.
Each entry has a shorthand like `HEAD@{3}` — meaning "where HEAD was 3 moves
ago". The reflog is local to your machine and typically expires after 90 days.

When you do something drastic — like `git reset --hard` or delete a branch —
the commits don't actually disappear. They just become unreachable from any
branch. But the reflog still points to them. This is how you recover from
almost any mistake.


## Cherry-Picking Commits

Sometimes you want a single commit from another branch without merging
everything:

```
git cherry-pick <commit-hash>
```

This copies the changes from that specific commit and applies them as a new
commit on your current branch. The original commit stays where it was.

Cherry-pick is surgical — you pick exactly what you want. Useful when a
bugfix was made on a feature branch and you need it on main right now, but
the rest of the feature isn't ready.


## Git Stash: Pause Your Work

You're mid-feature when something urgent comes up. Your changes aren't ready
to commit, but you need a clean working tree to switch branches.

```
git stash          # save changes, clean working tree
git stash list     # see all stashes
git stash pop      # restore most recent stash and remove it
git stash apply    # restore but keep the stash entry
```

Stash is a stack — you can stash multiple times, and `pop` pulls from the top.
Stashed changes include both staged and unstaged modifications. Untracked files
need the `-u` flag: `git stash -u`.

If you pop a stash and it conflicts with changes you've made since, git will
leave you in a conflict state. Resolve the conflicts just like a merge conflict.


## Detached HEAD

When you check out a specific commit hash instead of a branch name:

```
git checkout abc123
```

You enter **detached HEAD** state. HEAD now points directly at a commit, not
at a branch. You can look around, even make commits — but those commits won't
belong to any branch. If you switch away, they become unreachable (though still
in the reflog).

To keep work done in detached HEAD:

```
git checkout -b new-branch-name
```

This creates a branch at your current position, anchoring your commits.


## Rewriting History

Sometimes your last few commits are messy — typos, fixups, "oops forgot this
file" commits. You can squash them into a single clean commit:

```
git reset --soft HEAD~3    # undo last 3 commits, keep changes staged
git commit -m "clean combined commit"
```

The `--soft` flag is key: it moves HEAD back but leaves all your changes in
the staging area, ready to be re-committed as one.

This rewrites history — the original commits get new hashes (and the old ones
become reflog-only). Only do this on commits you haven't shared with others.

Another approach is interactive rebase (`git rebase -i HEAD~3`), which gives
you fine-grained control: squash, reorder, edit, or drop individual commits.
But `reset --soft` is simpler when you just want to combine everything.
