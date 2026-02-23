# Pipes & Redirection

Every command has three standard channels: stdin (input), stdout (output), and stderr (errors). Redirection and pipes let you wire these channels together to build powerful data pipelines.

## Standard Streams

| Stream  | Number | Default  | Purpose            |
|---------|--------|----------|--------------------|
| stdin   | 0      | keyboard | Input to command   |
| stdout  | 1      | terminal | Normal output      |
| stderr  | 2      | terminal | Error messages     |

## Output Redirection: `>` and `>>`

**`>`** writes stdout to a file (overwrites):

```bash
echo "hello" > greeting.txt     # creates/overwrites file
ls /nonexistent > out.txt       # stdout goes to file, errors still on terminal
```

**`>>`** appends to a file:

```bash
echo "line 1" > log.txt        # creates file
echo "line 2" >> log.txt       # appends to file
```

## Input Redirection: `<`

`<` sends a file's contents as stdin:

```bash
wc -l < data.txt               # count lines without filename in output
sort < names.txt                # sort the lines
```

## Pipes: `|`

A pipe connects one command's stdout to the next command's stdin:

```bash
ls | wc -l                     # count files
cat log.txt | grep "error"     # find error lines
ps aux | grep "python"         # find python processes
```

You can chain multiple pipes:

```bash
cat data.csv | grep "2024" | sort | head -5
```

Each `|` creates a new pipe: the left command writes, the right command reads.

## Redirecting stderr: `2>`

```bash
ls /nonexistent 2> errors.txt        # stderr to file
ls /nonexistent 2>/dev/null          # discard errors
ls /tmp /nonexistent > out.txt 2> err.txt  # separate stdout and stderr
```

Combine stdout and stderr:

```bash
command > all.txt 2>&1               # both to same file
command &> all.txt                   # bash shorthand (same thing)
```

## tee: Split Output

`tee` writes to both a file AND stdout:

```bash
echo "logged" | tee output.txt      # prints AND saves
ls | tee filelist.txt | wc -l       # saves list AND counts
```

## Command Substitution: `$()`

Capture a command's output as a string:

```bash
TODAY=$(date +%Y-%m-%d)
echo "Today is $TODAY"

FILES=$(ls | wc -l)
echo "There are $FILES files"
```

The older backtick syntax `` `command` `` works too but `$()` is preferred because it nests cleanly.

## /dev/null: The Bit Bucket

`/dev/null` is a special file that discards everything written to it:

```bash
command > /dev/null 2>&1     # silence all output
command &> /dev/null         # bash shorthand
```

## Key Takeaways

- `>` overwrites, `>>` appends, `<` provides input
- `|` pipes stdout of one command to stdin of the next
- `2>` redirects stderr separately, `2>&1` merges stderr into stdout
- `tee` splits output to both a file and the next command
- `$()` captures command output as a string
- `/dev/null` discards unwanted output
