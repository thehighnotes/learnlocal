# Text Processing

Linux excels at processing text. These tools let you sort, filter, transform, and extract data from text streams — the backbone of shell scripting.

## Sorting with `sort`

```bash
sort names.txt                  # alphabetical sort
sort -n numbers.txt             # numeric sort
sort -r names.txt               # reverse order
sort -t',' -k2 data.csv         # sort by 2nd comma-separated field
sort -u names.txt               # sort and remove duplicates
```

By default, sort compares lines as strings. Use `-n` for numbers, `-k` to sort by a specific field.

## Removing Duplicates with `uniq`

`uniq` removes **consecutive** duplicate lines — so always sort first:

```bash
sort data.txt | uniq            # remove duplicates
sort data.txt | uniq -c         # count occurrences
sort data.txt | uniq -d         # show only duplicates
```

## Extracting Fields with `cut`

`cut` extracts columns from structured text:

```bash
cut -d',' -f1 data.csv          # first comma-separated field
cut -d':' -f1,3 /etc/passwd     # fields 1 and 3, colon-delimited
cut -c1-5 file.txt              # first 5 characters of each line
```

The `-d` flag sets the delimiter, `-f` selects field numbers.

## Find and Replace with `sed`

`sed` (stream editor) transforms text:

```bash
sed 's/old/new/' file.txt            # replace first occurrence per line
sed 's/old/new/g' file.txt           # replace all occurrences
sed '3d' file.txt                    # delete line 3
sed -n '2,4p' file.txt              # print only lines 2-4
```

The `s` command is the most common: `s/pattern/replacement/flags`

## Column Extraction with `awk`

`awk` processes text field by field:

```bash
awk '{print $1}' file.txt            # print first field (space-delimited)
awk -F',' '{print $2}' data.csv      # second field, comma-delimited
awk '{print $1, $3}' file.txt        # print fields 1 and 3
awk '$3 > 100' data.txt              # lines where field 3 > 100
```

`$0` is the whole line, `$1` is the first field, `$NF` is the last field.

## Character Translation with `tr`

`tr` translates or deletes characters:

```bash
echo "hello" | tr 'a-z' 'A-Z'       # HELLO (lowercase to uppercase)
echo "hello" | tr -d 'l'             # heo (delete all 'l')
echo "a  b  c" | tr -s ' '          # a b c (squeeze repeated spaces)
```

`tr` works on characters, not words or patterns.

## Putting It Together

These tools combine powerfully:

```bash
# Count unique words in a file
cat text.txt | tr ' ' '\n' | sort | uniq -c | sort -rn | head -5

# Extract and sort email domains
cut -d'@' -f2 emails.txt | sort -u

# CSV: sum of second column
awk -F',' '{sum += $2} END {print sum}' data.csv
```

## Key Takeaways

- `sort` orders lines, `-n` for numeric, `-k` for specific fields
- `uniq` removes consecutive duplicates (sort first!), `-c` counts
- `cut -d -f` extracts fields from delimited text
- `sed 's/old/new/g'` does find-and-replace
- `awk '{print $N}'` extracts columns, supports conditions and math
- `tr` translates single characters, great for case conversion
