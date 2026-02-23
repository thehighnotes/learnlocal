# Goroutines & Channels

Go was designed with concurrency as a first-class feature. Where other languages
bolt on threads and locks as an afterthought, Go builds concurrency into the
language itself with two core primitives: **goroutines** and **channels**.

The philosophy is simple: *Do not communicate by sharing memory; instead, share
memory by communicating.*

## Goroutines

A **goroutine** is a lightweight thread of execution managed by the Go runtime.
You launch one by putting the `go` keyword before a function call:

```go
func sayHello() {
    fmt.Println("hello from goroutine")
}

func main() {
    go sayHello()  // launches a goroutine
    // main continues immediately without waiting
}
```

There is a problem with that code: `main()` might exit before the goroutine has
a chance to run. Goroutines are not automatically waited on. If `main` returns,
all goroutines are killed immediately.

You need a way to synchronize. The most idiomatic way in Go is **channels**.

## Channels

A **channel** is a typed conduit through which goroutines send and receive values.
Create one with `make`:

```go
ch := make(chan string)  // unbuffered channel of strings
```

Sending and receiving use the `<-` operator:

```go
ch <- "hello"    // send a value into the channel
msg := <-ch      // receive a value from the channel
```

An **unbuffered channel** blocks the sender until a receiver is ready, and vice
versa. This blocking behavior is what makes channels useful for synchronization:

```go
func greet(ch chan string) {
    ch <- "hello from goroutine"  // send blocks until main receives
}

func main() {
    ch := make(chan string)
    go greet(ch)           // launch goroutine
    msg := <-ch            // blocks until goroutine sends
    fmt.Println(msg)       // hello from goroutine
}
```

This pattern replaces `time.Sleep` hacks. The channel acts as both a
communication mechanism and a synchronization point.

### Sending Multiple Values

A goroutine can send multiple values on a channel. The receiver reads them
one at a time:

```go
func produce(ch chan int) {
    ch <- 1
    ch <- 2
    ch <- 3
}

func main() {
    ch := make(chan int)
    go produce(ch)
    fmt.Println(<-ch)  // 1
    fmt.Println(<-ch)  // 2
    fmt.Println(<-ch)  // 3
}
```

Each receive blocks until the next value is available. The order is guaranteed:
channels are FIFO (first in, first out).

## Buffered Channels

By default, channels are **unbuffered** -- a send blocks until something receives.
A **buffered channel** has internal capacity. Sends only block when the buffer is
full, and receives only block when the buffer is empty:

```go
ch := make(chan string, 2)  // buffer capacity of 2

ch <- "first"   // does not block (buffer has room)
ch <- "second"  // does not block (buffer has room)
// ch <- "third" would block here -- buffer is full

fmt.Println(<-ch)  // "first"
fmt.Println(<-ch)  // "second"
```

Buffered channels are useful when you know how many values will be produced, or
when you want to decouple the speed of sender and receiver.

A common pattern is to buffer a channel to the number of results you expect:

```go
ch := make(chan int, 3)
ch <- 10
ch <- 20
ch <- 30
close(ch)

for val := range ch {
    fmt.Println(val)
}
```

The `close()` function signals that no more values will be sent. The `range` loop
reads until the channel is closed.

## The select Statement

The `select` statement lets a goroutine wait on multiple channel operations at
once. It looks like a `switch`, but each case is a channel send or receive:

```go
select {
case msg := <-ch1:
    fmt.Println("received from ch1:", msg)
case msg := <-ch2:
    fmt.Println("received from ch2:", msg)
}
```

If multiple cases are ready, `select` picks one **at random**. If none are ready,
it blocks until one becomes ready.

### Deterministic select

When you need deterministic behavior, you control which channels have data and
when. For example, you can send values to channels sequentially before selecting:

```go
ch1 := make(chan string, 1)
ch2 := make(chan string, 1)

ch1 <- "ping"

select {
case msg := <-ch1:
    fmt.Println(msg)  // always "ping" -- ch1 is the only ready channel
case msg := <-ch2:
    fmt.Println(msg)
}
```

By buffering the channels and controlling when you send, you control which case
the `select` picks.

## sync.WaitGroup

When you have multiple goroutines and want to wait for all of them to finish,
use `sync.WaitGroup`:

```go
var wg sync.WaitGroup

for i := 1; i <= 3; i++ {
    wg.Add(1)           // increment counter before launching
    go func(n int) {
        defer wg.Done() // decrement counter when goroutine finishes
        fmt.Printf("worker %d done\n", n)
    }(i)
}

wg.Wait()  // blocks until counter reaches zero
```

The three methods:
- `Add(n)` -- increment the counter by n (call before `go`)
- `Done()` -- decrement the counter by 1 (call inside the goroutine)
- `Wait()` -- block until the counter reaches zero

**Important:** Always call `Add()` *before* launching the goroutine, not inside
it. Otherwise there is a race between `Wait()` seeing zero and `Add()` incrementing.

### Ordering Goroutine Output

Goroutines run concurrently, so their output order is not guaranteed. If you need
deterministic output, use a channel to collect results and print in order:

```go
var wg sync.WaitGroup
results := make([]string, 3)

for i := 0; i < 3; i++ {
    wg.Add(1)
    go func(n int) {
        defer wg.Done()
        results[n] = fmt.Sprintf("worker %d done", n+1)
    }(i)
}

wg.Wait()
for _, r := range results {
    fmt.Println(r)
}
```

Each goroutine writes to its own index in the slice, so there are no races. After
`Wait()` returns, the results are printed in order.

## sync.Mutex

When multiple goroutines need to read or write the same variable, you have a
**data race**. Go provides `sync.Mutex` (mutual exclusion lock) to protect
shared state:

```go
var (
    mu      sync.Mutex
    counter int
)

func increment(wg *sync.WaitGroup) {
    defer wg.Done()
    mu.Lock()
    counter++
    mu.Unlock()
}

func main() {
    var wg sync.WaitGroup
    for i := 0; i < 1000; i++ {
        wg.Add(1)
        go increment(&wg)
    }
    wg.Wait()
    fmt.Printf("count=%d\n", counter)  // always 1000
}
```

Without the mutex, the counter would have unpredictable results due to concurrent
read-modify-write operations. The mutex ensures only one goroutine accesses the
counter at a time.

### Lock and Unlock

- `mu.Lock()` -- acquire the lock. If another goroutine holds it, this blocks.
- `mu.Unlock()` -- release the lock. Another blocked goroutine can now proceed.

A common pattern is `defer mu.Unlock()` right after `Lock()`:

```go
mu.Lock()
defer mu.Unlock()
// ... safe to read/write shared state ...
```

This ensures the lock is always released, even if the function panics.

### Concurrency vs Parallelism

A quick note on terminology:

- **Concurrency** is about *structure* -- designing your program to handle
  multiple tasks that can make progress independently.
- **Parallelism** is about *execution* -- actually running multiple tasks at the
  same time on multiple CPU cores.

Go gives you concurrency with goroutines. Whether they run in parallel depends
on the number of CPU cores and the `GOMAXPROCS` setting (which defaults to the
number of cores).

## Testing Go Code

Go has built-in testing that requires no external framework. Test files end in
`_test.go` and test functions start with `Test`:

```go
// calculator_test.go
package main

import "testing"

func TestAdd(t *testing.T) {
    if Add(2, 3) != 5 {
        t.Error("Add(2, 3) should be 5")
    }
}
```

Run tests with:

```bash
go test         # runs all tests in the current directory
go test -v      # verbose output — shows each test name and result
```

Go discovers test files and functions automatically by their naming convention.
The `testing.T` parameter provides methods like `t.Error()` and `t.Fatal()` for
reporting failures.

In the exercises that follow, you will practice launching goroutines, communicating
through channels, using buffered channels and select, coordinating with WaitGroup,
and protecting shared state with Mutex.
