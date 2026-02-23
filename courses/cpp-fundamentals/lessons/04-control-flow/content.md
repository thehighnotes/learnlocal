# Control Flow

So far, your programs have executed line by line from top to bottom. Control flow
statements let you change that: skip lines, choose between alternatives, or repeat
a block multiple times.

## if / else

The `if` statement executes a block only when a condition is true:

```cpp
int temperature = 35;

if (temperature > 30) {
    std::cout << "It's hot outside!" << std::endl;
}
```

Add `else` to handle the false case:

```cpp
if (temperature > 30) {
    std::cout << "Hot" << std::endl;
} else {
    std::cout << "Not hot" << std::endl;
}
```

Chain conditions with `else if`:

```cpp
if (temperature > 30) {
    std::cout << "Hot" << std::endl;
} else if (temperature > 15) {
    std::cout << "Mild" << std::endl;
} else {
    std::cout << "Cold" << std::endl;
}
```

The condition inside the parentheses must evaluate to a boolean (`true` or `false`).
Comparison operators like `>`, `<`, `>=`, `<=`, `==`, and `!=` produce boolean results.

## switch

When you need to compare a single value against many options, `switch` is cleaner
than a long chain of `else if`:

```cpp
int day = 3;

switch (day) {
    case 1:
        std::cout << "Monday" << std::endl;
        break;
    case 2:
        std::cout << "Tuesday" << std::endl;
        break;
    case 3:
        std::cout << "Wednesday" << std::endl;
        break;
    default:
        std::cout << "Other day" << std::endl;
        break;
}
```

Each `case` label marks a possible value. The `break` statement exits the switch
after a match. Without `break`, execution "falls through" to the next case -- this
is a common source of bugs. The `default` case handles any value not explicitly
listed.

## while Loops

A `while` loop repeats a block as long as its condition remains true:

```cpp
int count = 1;
while (count <= 3) {
    std::cout << count << std::endl;
    count++;
}
```

This prints 1, 2, 3. The variable `count` is updated each iteration. If you forget
to update it, the loop runs forever -- an infinite loop.

## for Loops

A `for` loop packs initialization, condition, and update into one line:

```cpp
for (int i = 0; i < 5; i++) {
    std::cout << i << std::endl;
}
```

The three parts separated by semicolons are:
1. **Initialization:** `int i = 0` -- runs once before the loop starts
2. **Condition:** `i < 5` -- checked before each iteration
3. **Update:** `i++` -- runs after each iteration

This is equivalent to a `while` loop with the same logic, but keeps the loop
control in one place.

## do-while Loops

A `do-while` loop is like a `while` loop, but it checks the condition *after*
the body executes. This guarantees the body runs at least once:

```cpp
int n = 1;
do {
    std::cout << n << std::endl;
    n *= 2;
} while (n <= 100);
```

Notice the semicolon after `while(...)` -- this is required and easy to forget.

The `do-while` is useful when you need at least one execution before checking
whether to continue. A common real-world use is input validation:

```cpp
int choice;
do {
    std::cout << "Enter 1, 2, or 3: ";
    std::cin >> choice;
} while (choice < 1 || choice > 3);
```

This keeps asking until the user enters a valid value. A regular `while` loop
would need you to either duplicate the prompt or initialize the variable to an
invalid value.

## break and continue

Two keywords give you finer control inside any loop:

**`break`** exits the loop immediately, skipping any remaining iterations:

```cpp
for (int i = 1; i <= 10; i++) {
    if (i == 5) {
        break;  // stop at 5
    }
    std::cout << i << std::endl;
}
// Prints: 1 2 3 4 (each on its own line)
```

**`continue`** skips the rest of the current iteration and jumps to the next one:

```cpp
for (int i = 1; i <= 5; i++) {
    if (i == 3) {
        continue;  // skip 3
    }
    std::cout << i << std::endl;
}
// Prints: 1 2 4 5 (each on its own line)
```

When `break` and `continue` are used together, order matters. Check skip
conditions first, then check stop conditions, or vice versa -- depending on the
logic you need.

## Nested Loops

You can place one loop inside another. The inner loop runs completely for each
iteration of the outer loop:

```cpp
for (int row = 1; row <= 3; row++) {
    for (int col = 1; col <= 3; col++) {
        std::cout << "(" << row << "," << col << ") ";
    }
    std::cout << std::endl;
}
```

This prints a 3x3 grid of coordinates. The outer loop controls the rows, the
inner loop controls the columns. For each row, the inner loop runs all its
iterations before the outer loop advances.

Nested loops are essential for working with 2D structures -- grids, tables,
patterns, and matrices. The total number of iterations is the product of the
two loop counts (3 x 3 = 9 here).

A common pattern is using the outer loop variable to control the inner loop's
range:

```cpp
for (int i = 1; i <= 4; i++) {
    for (int j = 0; j < i; j++) {
        std::cout << "*";
    }
    std::cout << std::endl;
}
```

This prints a triangle because each row has as many stars as its row number.

## FizzBuzz: Loops Meet Conditionals

FizzBuzz is a classic programming exercise that combines a loop with conditional
logic. The rules: for numbers 1 to N, print "Fizz" for multiples of 3, "Buzz"
for multiples of 5, "FizzBuzz" for multiples of both, and the number itself
otherwise.

The key insight is checking the "both" case first:

```cpp
if (i % 3 == 0 && i % 5 == 0) {
    std::cout << "FizzBuzz";
} else if (i % 3 == 0) {
    std::cout << "Fizz";
} else if (i % 5 == 0) {
    std::cout << "Buzz";
} else {
    std::cout << i;
}
```

If you checked `i % 3 == 0` first, numbers like 15 would print "Fizz" and
never reach the "FizzBuzz" case. Order of conditions matters.

## Choosing the Right Tool

- Use `if/else` for simple yes/no decisions.
- Use `switch` when comparing one value against several constants.
- Use `while` when you don't know in advance how many times to loop.
- Use `for` when you know the number of iterations (or are iterating a range).
- Use `do-while` when the body must execute at least once.
- Use `break` to exit a loop early when a condition is met.
- Use `continue` to skip specific iterations without stopping the loop.
- Use nested loops when working with 2D patterns or grids.
