# Control Flow

Programs that only run top to bottom, one statement after another, can't do much. Real programs need to make decisions — should I show an error or a success message? They need to repeat actions — process every item in a list, retry until something works. Control flow is how you tell JavaScript *when* to run code and *how many times*.

## if and else

The `if` statement is the most fundamental decision-making tool. It runs a block of code only when a condition is true:

```javascript
let temperature = 35;

if (temperature > 30) {
  console.log("It's hot outside!");
}
```

The condition goes in parentheses. The code to run goes in curly braces. If the condition is false, the block is simply skipped.

Add `else` to handle the false case:

```javascript
let age = 16;

if (age >= 18) {
  console.log("You can vote");
} else {
  console.log("Too young to vote");
}
```

For single-statement bodies, braces are technically optional — but always use them. It prevents a whole category of bugs when you later add a second line:

```javascript
// Dangerous — only the first line is conditional
if (loggedIn)
  console.log("Welcome back");
  showDashboard(); // This ALWAYS runs, despite the indentation!

// Safe — both lines are clearly inside the if
if (loggedIn) {
  console.log("Welcome back");
  showDashboard();
}
```

JavaScript conditions work with truthy and falsy values. These are falsy: `false`, `0`, `""` (empty string), `null`, `undefined`, and `NaN`. Everything else is truthy, including `"0"`, `[]`, and `{}`.

## else if Chains

When you have more than two possibilities, chain `else if` clauses:

```javascript
let score = 85;

if (score >= 90) {
  console.log("A");
} else if (score >= 80) {
  console.log("B");
} else if (score >= 70) {
  console.log("C");
} else if (score >= 60) {
  console.log("D");
} else {
  console.log("F");
}
```

Order matters. Conditions are checked top to bottom, and the first one that matches wins. Because we check `>= 90` first, by the time we reach `>= 80` we already know the score is below 90. This lets us write simpler conditions — no need for `score >= 80 && score < 90`.

If you reverse the order, everything would match the first condition:

```javascript
// Bug: score 95 would print "D" because 95 >= 60 is true
if (score >= 60) {
  console.log("D"); // This catches almost everything!
}
```

## The switch Statement

When comparing one value against many specific cases, `switch` can be cleaner than a long if/else chain:

```javascript
let fruit = "apple";

switch (fruit) {
  case "apple":
    console.log("$1.50 per pound");
    break;
  case "banana":
    console.log("$0.75 per pound");
    break;
  case "cherry":
    console.log("$4.00 per pound");
    break;
  default:
    console.log("Fruit not in catalog");
}
```

The `break` statement is critical. Without it, execution "falls through" to the next case. This is a feature, not a bug — sometimes you want it:

```javascript
let day = "Saturday";

switch (day) {
  case "Saturday":
  case "Sunday":
    console.log("Weekend!"); // Both cases share this code
    break;
  default:
    console.log("Weekday");
}
```

But accidental fall-through is one of the most common switch bugs. Always include `break` unless you deliberately want fall-through.

The `default` case runs when no other case matches. It's like the `else` at the end of an if/else chain. While optional, including it makes your intent clear.

Use `switch` when comparing a single value against discrete options. Use `if/else` when conditions involve ranges, complex logic, or different variables.

## while Loops

A `while` loop repeats code as long as a condition remains true:

```javascript
let count = 1;

while (count <= 5) {
  console.log(count);
  count++;
}
// Output: 1, 2, 3, 4, 5
```

The condition is checked before each iteration. If it's false from the start, the body never runs at all.

Be careful: if the condition never becomes false, you get an infinite loop and your program hangs:

```javascript
// DANGER: infinite loop — count never changes
let count = 1;
while (count <= 5) {
  console.log(count);
  // Forgot count++!
}
```

You can exit a loop early with `break`:

```javascript
let num = 1;

while (true) {
  if (num * num > 50) {
    break; // Exit when the square exceeds 50
  }
  console.log(num);
  num++;
}
```

## for Loops

The `for` loop packs initialization, condition, and update into one line. This makes it ideal when you know how many times to iterate:

```javascript
for (let i = 0; i < 5; i++) {
  console.log(i);
}
// Output: 0, 1, 2, 3, 4
```

The three parts are separated by semicolons:
- **Init** (`let i = 0`): runs once before the loop starts
- **Condition** (`i < 5`): checked before each iteration
- **Update** (`i++`): runs after each iteration

You can count by different increments:

```javascript
// Count by 2
for (let i = 0; i <= 10; i += 2) {
  console.log(i); // 0, 2, 4, 6, 8, 10
}

// Count backwards
for (let i = 10; i > 0; i--) {
  console.log(i); // 10, 9, 8, ... 1
}
```

## break and continue

You've seen `break` exit a loop entirely. The `continue` statement skips the rest of the current iteration and jumps to the next one:

```javascript
for (let i = 1; i <= 10; i++) {
  if (i % 2 === 0) {
    continue; // Skip even numbers
  }
  console.log(i); // Only prints odd: 1, 3, 5, 7, 9
}
```

With nested loops, `break` and `continue` only affect the innermost loop. If you need to break out of an outer loop, you can use a labeled statement:

```javascript
outer: for (let i = 0; i < 3; i++) {
  for (let j = 0; j < 3; j++) {
    if (i === 1 && j === 1) {
      break outer; // Breaks out of BOTH loops
    }
    console.log(i, j);
  }
}
```

Labeled breaks are rarely needed. If you find yourself reaching for one, consider refactoring into a function instead.

## for...of Loops

The `for...of` loop iterates directly over values in an iterable (strings, arrays, and more):

```javascript
let word = "Hello";

for (const char of word) {
  console.log(char);
}
// H, e, l, l, o
```

Compare this to a traditional index-based loop:

```javascript
// Traditional — you manage the index yourself
for (let i = 0; i < word.length; i++) {
  console.log(word[i]);
}

// for...of — cleaner when you just need the values
for (const char of word) {
  console.log(char);
}
```

Use `for...of` when you need the values. Use a traditional `for` loop when you need the index or need to skip/step through elements. You'll use `for...of` extensively with arrays in a later lesson.

## Nested Loops

A loop inside another loop creates combinations. The inner loop runs completely for each iteration of the outer loop:

```javascript
for (let row = 1; row <= 3; row++) {
  for (let col = 1; col <= 3; col++) {
    console.log(`${row} x ${col} = ${row * col}`);
  }
}
```

This produces 9 lines (3 rows times 3 columns). Nested loops are used for:
- Multiplication tables and grids
- Comparing every pair of items
- Working with 2D data (rows and columns)
- Generating patterns

Be mindful of performance: nested loops multiply. A loop of 1000 inside a loop of 1000 means 1,000,000 iterations.

## Putting It All Together

FizzBuzz is the classic exercise that combines loops with conditionals. For numbers 1 to 15: print "Fizz" for multiples of 3, "Buzz" for multiples of 5, "FizzBuzz" for multiples of both, and the number itself otherwise:

```javascript
for (let i = 1; i <= 15; i++) {
  if (i % 3 === 0 && i % 5 === 0) {
    console.log("FizzBuzz");
  } else if (i % 3 === 0) {
    console.log("Fizz");
  } else if (i % 5 === 0) {
    console.log("Buzz");
  } else {
    console.log(i);
  }
}
```

The key insight: check the combined condition first. If you check `i % 3 === 0` before `i % 3 === 0 && i % 5 === 0`, then 15 would match "Fizz" and never reach "FizzBuzz". The else if chain means only the first matching branch runs.
