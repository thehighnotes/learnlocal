# Branching & Merging

Branches are git's killer feature. They let you work on multiple things at once
without stepping on your own toes. A branch is just a lightweight pointer to a
commit -- creating one is nearly instantaneous and costs almost nothing.

## What Is a Branch?

Every commit in git points back to its parent (or parents, for merges). A
**branch** is simply a named pointer to one of those commits. When you make a
new commit on a branch, the pointer moves forward automatically.

```
main ──> C1 ──> C2 ──> C3
```

Creating a branch called `feature` from `C3` just adds a second pointer:

```
main ──> C1 ──> C2 ──> C3 <── feature
```

Both `main` and `feature` point to the same commit. They cost nothing extra
until you start adding commits to one of them.

## HEAD: Where You Are Right Now

Git tracks which branch you're "on" with a special pointer called **HEAD**.
When HEAD points to `main`, new commits advance `main`. When HEAD points to
`feature`, new commits advance `feature`.

```
         HEAD
          |
main ──> C1 ──> C2 ──> C3 <── feature
```

## Creating Branches

```bash
git branch feature          # create branch (stay where you are)
git switch -c feature       # create AND switch in one step
git checkout -b feature     # older syntax, same effect
```

`git branch` just creates the pointer. You still need to switch to it before
your commits land there.

## Switching Branches

```bash
git switch feature          # modern way
git checkout feature        # classic way
```

Both do the same thing: move HEAD to point at the named branch and update
your working directory to match that branch's latest commit.

## Listing Branches

```bash
git branch                  # list local branches (* marks current)
git branch -v               # include last commit message
git branch --merged         # branches already merged into current
git branch --no-merged      # branches with unmerged work
```

## Merging

When you're done with a feature, you merge it back. Git has two main merge strategies:

### Fast-Forward Merge

If the target branch hasn't diverged, git just moves the pointer forward:

```
Before:
main ──> C1 ──> C2
                     \
                      C3 ──> C4 <── feature

After `git merge feature` (from main):
main ──> C1 ──> C2 ──> C3 ──> C4 <── feature
                                 ^
                                main
```

No merge commit is created. The history stays linear.

### Three-Way Merge

If both branches have new commits, git creates a **merge commit** with two
parents:

```
Before:
main ──> C1 ──> C2 ──> C5
                     \
                      C3 ──> C4 <── feature

After `git merge feature` (from main):
main ──> C1 ──> C2 ──> C5 ──> M <── main
                     \            /
                      C3 ──> C4 <── feature
```

The merge commit `M` has two parents: `C5` and `C4`. Git automatically
combines the changes as long as they don't conflict.

### Running a Merge

```bash
git switch main             # go to the branch you want to merge INTO
git merge feature           # bring feature's changes into main
```

Always switch to the receiving branch first. "Merge X into Y" means you're
on Y and you run `git merge X`.

## Deleting Branches

Once a branch is merged, the pointer is just clutter. Clean it up:

```bash
git branch -d feature       # delete (only if merged)
git branch -D feature       # force delete (even if unmerged)
```

`-d` is the safe option: git refuses if the branch has unmerged work. `-D`
is the "I know what I'm doing" override.

## Branching from a Specific Commit

You can create a branch from any commit, not just the current one:

```bash
git branch hotfix abc1234   # branch from a specific commit hash
git switch -c hotfix abc1234  # create and switch in one step
```

This is useful for hotfixes: branch from the last release commit instead of
from the current development tip.

## Why Branches Are Cheap

In many version control systems, creating a branch copies the entire
repository. In git, a branch is literally a 41-byte file containing a commit
hash. Creating 100 branches costs about 4KB of disk space.

This cheapness changes how you work. Instead of one long-lived branch with
careful commits, you create branches freely:
- Feature branches for new work
- Bugfix branches for fixes
- Experiment branches you might throw away

Branch early, branch often, merge when ready, delete when done.
