# Conflict Resolution

Merge conflicts happen when Git can't automatically combine changes from two branches.
They're not errors — they're Git asking for your help.

## What Causes a Conflict?

A conflict occurs when two branches change the **same lines** in the **same file**.

```
main:     "Hello World"  →  "Hello Earth"
feature:  "Hello World"  →  "Hello Mars"
```

Git can merge changes in **different** files, or even **different lines** of the same file,
automatically. But when two branches touch the same line, Git stops and asks you to decide.

## Conflict Markers

When a merge hits a conflict, Git writes special markers into the file:

```
<<<<<<< HEAD
Hello Earth
=======
Hello Mars
>>>>>>> feature
```

Three markers, three meanings:

| Marker            | Meaning                              |
|-------------------|--------------------------------------|
| `<<<<<<< HEAD`    | Start of YOUR version (current branch) |
| `=======`         | Divider between the two versions     |
| `>>>>>>> feature` | End of THEIR version (incoming branch) |

Everything between `<<<<<<<` and `=======` is what's on your current branch.
Everything between `=======` and `>>>>>>>` is what's coming from the other branch.

## Resolving a Conflict

To resolve a conflict, you:

1. Open the file with conflict markers
2. Decide what the final content should be
3. Remove ALL conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`)
4. Stage the file with `git add`
5. Complete the merge with `git commit`

You can keep one side, the other side, both, or write something entirely new.
The only rule: **no conflict markers can remain**.

## Checking for Conflicts

```bash
git status              # Shows "both modified" for conflicted files
git diff                # Shows the conflict markers in context
```

## Aborting a Merge

Changed your mind? Not ready to deal with conflicts right now?

```bash
git merge --abort       # Cancels the merge, returns to pre-merge state
```

This is always safe. It restores your working directory to exactly how it was
before you ran `git merge`.

## Resolution Strategies

Sometimes you know in advance which side should win:

```bash
# Keep our version for all conflicts
git checkout --ours <file>
git add <file>

# Keep their version for all conflicts
git checkout --theirs <file>
git add <file>
```

Or at merge time:

```bash
git merge -s ours feature      # Keep main's version entirely
```

## Creative Resolution

You're not limited to picking one side. You can combine both versions,
rewrite the line entirely, or come up with a third option. A merge conflict
is just Git asking "what should this look like?" — and you're the answer.

## Key Takeaways

- Conflicts happen when two branches edit the same lines
- Conflict markers show both versions separated by `=======`
- Remove ALL markers and decide the final content
- `git merge --abort` safely cancels a conflicted merge
- `--ours` and `--theirs` pick sides without manual editing
- You can always write a third option that combines or replaces both
