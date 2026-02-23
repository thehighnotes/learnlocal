## Welcome to the Danger Zone

Everything you've learned so far — branching, merging, undoing, the reflog,
cherry-pick, stash — it all comes together here. These are real-world git
disasters that every developer hits eventually. Some of them will feel
impossible at first. They're not.


## Force Push Recovery

The most feared git disaster: someone runs `git push --force` and overwrites
the remote history. Your carefully crafted commits just got replaced.

But if you had the branch checked out locally, your reflog still has the old
commits. The recovery pattern:

```
git reflog                           # find your lost commits
git branch recovery <old-hash>       # create a branch at the old position
```

This is why you always pull before force-pushing, and why `--force-with-lease`
exists (it refuses to push if the remote has commits you haven't seen).


## Git Bisect: Finding the Needle

Your app was working last Tuesday. Now it's broken. Somewhere in the last 50
commits, something went wrong. Checking each one manually would take hours.

`git bisect` does a binary search through your commit history:

```
git bisect start
git bisect bad                 # current commit is broken
git bisect good <known-good>   # this commit was fine
# git checks out the middle commit
# you test it, then tell git:
git bisect good    # or
git bisect bad
# repeat until git finds the first bad commit
git bisect reset   # when done
```

Bisect halves the search space each time. 50 commits? About 6 tests. 1000
commits? About 10 tests. It's logarithmic — devastatingly efficient.

You can even automate it with a test script:

```
git bisect start HEAD <good-hash>
git bisect run ./test.sh
```


## Untangling Commits

Sometimes you realize the last commit mixed two unrelated changes. Good commit
hygiene says each commit should be a single logical unit. Fix it:

```
git reset --soft HEAD~1   # undo the commit, keep changes staged
git reset HEAD file1.txt  # unstage file1
git commit -m "change to file2 only"
git add file1.txt
git commit -m "change to file1 only"
```

Now you have two clean commits instead of one messy one.


## Moving Commits Between Branches

Committed on the wrong branch? This happens all the time:

```
# Save the commit hash
git log --oneline -1

# Move to the right branch
git checkout correct-branch
git cherry-pick <hash>

# Go back and remove from wrong branch
git checkout wrong-branch
git reset --hard HEAD~1
```

Cherry-pick copies the commit. Reset removes it from the wrong branch. Clean.


## File Archaeology

Need to recover an old version of a file? Git stores every version:

```
git log -- path/to/file         # see commit history for this file
git show <hash>:path/to/file    # view the file at that commit
git checkout <hash> -- path/to/file   # restore that version
```

The `--` separator tells git "everything after this is a file path, not a
branch name." Use it whenever there's ambiguity.


## Complex Merges

When multiple branches all touch the same code, merging gets messy. The key
principle: merge one branch at a time, resolve conflicts fully, commit, then
merge the next. Don't try to merge everything at once.

```
git merge branch-a             # resolve conflicts if any
git add . && git commit         # if there were conflicts
git merge branch-b             # resolve these conflicts too
git add . && git commit         # finalize
```

After resolving conflicts, always check that no conflict markers (`<<<<<<<`,
`=======`, `>>>>>>>`) remain in any file. A stray marker means broken code.
