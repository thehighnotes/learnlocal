# Viewing & Searching

You know how to navigate and manage files. Now let's look inside them and find what you need.

## Viewing Files with `cat`, `head`, and `tail`

**`cat`** prints the entire file:

```bash
cat readme.txt             # print everything
cat file1.txt file2.txt    # concatenate and print both
```

**`head`** shows the first N lines (default 10):

```bash
head -5 logfile.txt        # first 5 lines
head -n 20 data.csv        # first 20 lines
```

**`tail`** shows the last N lines:

```bash
tail -3 logfile.txt        # last 3 lines
tail -f /var/log/syslog    # follow — watch new lines appear live
```

## Searching with `grep`

`grep` finds lines matching a pattern:

```bash
grep "error" logfile.txt           # lines containing "error"
grep -i "warning" logfile.txt      # case-insensitive
grep -n "TODO" source.py           # show line numbers
grep -c "error" logfile.txt        # count matching lines
grep -r "import" src/              # recursive search in directory
```

The pattern can be a regular expression:

```bash
grep "^Start" file.txt      # lines starting with "Start"
grep "end$" file.txt         # lines ending with "end"
grep "[0-9]" file.txt        # lines containing a digit
```

## Finding Files with `find`

`find` searches the filesystem by name, type, size, and more:

```bash
find . -name "*.txt"              # find .txt files
find /home -type d -name "config" # find directories named config
find . -name "*.log" -mtime -7    # .log files modified in last 7 days
find . -type f -empty              # empty files
```

The output is one path per line, and the paths include the starting directory.

## Counting with `wc`

`wc` (word count) counts lines, words, and bytes:

```bash
wc file.txt                 # lines, words, bytes
wc -l file.txt              # just line count
wc -w file.txt              # just word count
wc -c file.txt              # just byte count
```

## Combining Tools

These tools work great together with pipes (which you'll learn in detail soon):

```bash
cat logfile.txt | grep "error" | wc -l    # count error lines
find . -name "*.py" | wc -l               # count Python files
head -20 data.csv | grep "alice"           # search in first 20 lines
```

## Key Takeaways

- `cat` for full files, `head -N` for the start, `tail -N` for the end
- `grep` searches for patterns — use `-i` for case-insensitive, `-n` for line numbers, `-c` for counts
- `find` searches the filesystem by name, type, and other attributes
- `wc -l` counts lines, `-w` counts words
