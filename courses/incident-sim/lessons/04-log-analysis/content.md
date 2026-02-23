# Log Analysis

Every incident leaves a trail in the logs. The challenge isn't finding logs — it's
finding the **right** lines among millions. A production server can generate gigabytes
of logs per day. Your job is to cut through the noise.


## Common Log Formats

**Application logs** typically look like:

```
2024-03-15 14:32:01 [INFO] Request processed in 12ms
2024-03-15 14:32:01 [ERROR] Database connection timeout after 30s
2024-03-15 14:32:02 [WARN] Retry attempt 3 of 5 for request abc-123
```

**HTTP access logs** (Apache/Nginx combined format):

```
192.168.1.50 - - [15/Mar/2024:14:32:01 +0000] "GET /api/users HTTP/1.1" 200 1234
10.0.0.15 - - [15/Mar/2024:14:32:02 +0000] "POST /api/login HTTP/1.1" 500 89
```

The fields: IP, identity, user, timestamp, request, status code, response size.


## grep — Your First Weapon

`grep` finds lines matching a pattern:

```bash
grep "ERROR" app.log              # lines containing ERROR
grep -i "error" app.log           # case-insensitive
grep -c "ERROR" app.log           # count matching lines
grep -n "ERROR" app.log           # show line numbers
grep -v "DEBUG" app.log           # exclude DEBUG lines
grep "ERROR\|FATAL" app.log       # ERROR or FATAL
```

For regex patterns, use `grep -E` (extended regex):

```bash
grep -E "5[0-9]{2}" access.log   # 500-599 status codes
grep -E "2024-03-15 14:3[0-5]" app.log  # time window
```


## awk — Extract Fields

`awk` splits lines into fields and lets you pick the ones you need:

```bash
awk '{print $1}' access.log       # first field (IP address)
awk '{print $9}' access.log       # ninth field (HTTP status)
awk -F'"' '{print $2}' access.log # split on quotes, get request
```

Combine with conditions:

```bash
awk '$9 == 500 {print $1, $7}' access.log  # IPs hitting 500 errors
```


## sort and uniq — Count and Rank

These two are almost always used together:

```bash
sort access.log | uniq -c | sort -rn    # count duplicate lines
awk '{print $1}' access.log | sort | uniq -c | sort -rn   # top IPs
awk '{print $9}' access.log | sort | uniq -c | sort -rn   # status code counts
```

The pipeline: extract field → sort → count unique → sort by count (descending).


## Time-Based Filtering

Incidents happen in time windows. Filter by timestamp:

```bash
grep "2024-03-15 14:3" app.log          # all of 14:30-14:39
awk '$1 == "2024-03-15" && $2 >= "14:30" && $2 <= "14:45"' app.log
sed -n '/14:30:00/,/14:45:00/p' app.log  # range between two patterns
```


## Cross-Referencing Logs

Real incidents span multiple services. The link between them is usually a
**request ID**, **transaction ID**, or **timestamp**:

```bash
# Find the request ID in the app log
grep "ERROR" app.log
# 2024-03-15 14:32:01 [ERROR] Failed processing request req-abc-789

# Search for that ID in other logs
grep "req-abc-789" db.log nginx.log queue.log
```

This is how you trace a failure through a distributed system.


## Key Takeaways

- `grep` to filter, `awk` to extract fields, `sort | uniq -c` to count
- Always start broad, then narrow: all errors → time window → specific pattern
- Request IDs are your best friend for cross-referencing logs
- Pipe commands together — each step reduces the data by an order of magnitude
- During an incident, build a **timeline** from log entries across all services
