# Functions

Functions are reusable blocks of code that perform a specific task. Instead of writing the same logic over and over, you wrap it in a function and call it whenever you need it. This is the DRY principle — Don't Repeat Yourself.

Functions are one of the most fundamental building blocks in JavaScript. They let you organize code into logical pieces, give those pieces meaningful names, and combine them to solve larger problems.

## Function Declarations

You create a function with the `function` keyword, followed by a name, parentheses for parameters, and a body in curly braces:

```javascript
function sayHello() {
  console.log("Hello!");
}

sayHello(); // prints: Hello!
sayHello(); // prints: Hello! (you can call it as many times as you want)
```

Function names follow the same rules as variable names — they should be descriptive and use camelCase by convention.

**Hoisting:** Function declarations are "hoisted" to the top of their scope. This means you can call a function before it appears in the code:

```javascript
greet(); // This works! prints: Hi there

function greet() {
  console.log("Hi there");
}
```

This only works with function declarations (the `function name() {}` form). Function expressions and arrow functions are **not** hoisted.

## Parameters and Arguments

Parameters let functions accept input. You list them inside the parentheses when defining the function. When calling the function, the values you pass in are called arguments:

```javascript
function greet(name) {
  console.log("Hello, " + name + "!");
}

greet("Alice"); // Hello, Alice!
greet("Bob");   // Hello, Bob!
```

Functions can take multiple parameters, separated by commas:

```javascript
function add(a, b) {
  console.log(a + b);
}

add(3, 5); // 8
```

A few things to know about JavaScript function arguments:

- **Extra arguments are ignored:** `add(3, 5, 99)` still works — 99 is simply unused.
- **Missing arguments become undefined:** `add(3)` sets `a` to 3 and `b` to `undefined`, so the result is `NaN` (Not a Number).
- Arguments are positional — the first argument maps to the first parameter, and so on.

## Return Values

Functions can send a value back to the caller using the `return` keyword:

```javascript
function add(a, b) {
  return a + b;
}

let sum = add(3, 5);
console.log(sum); // 8
```

Without `return`, a function returns `undefined`:

```javascript
function doSomething() {
  let x = 42; // computed but never returned
}

let result = doSomething();
console.log(result); // undefined
```

`return` also immediately exits the function — any code after it does not run:

```javascript
function absolute(x) {
  if (x < 0) {
    return -x; // exits here if x is negative
  }
  return x;    // only reached if x >= 0
}
```

A function can only return one value. If you need to return multiple values, wrap them in an array or object:

```javascript
function minMax(arr) {
  return { min: Math.min(...arr), max: Math.max(...arr) };
}

let result = minMax([3, 1, 7, 2]);
console.log(result.min); // 1
console.log(result.max); // 7
```

## Default Parameters

Default parameters provide fallback values when an argument is not passed (or is `undefined`):

```javascript
function greet(name = "World") {
  console.log(`Hello, ${name}!`);
}

greet();        // Hello, World!
greet("Alice"); // Hello, Alice!
```

Defaults are evaluated left to right, so later defaults can reference earlier parameters:

```javascript
function createUser(name, role = "member") {
  return { name, role };
}

createUser("Alice");          // { name: "Alice", role: "member" }
createUser("Bob", "admin");   // { name: "Bob", role: "admin" }
```

The default is only used when the argument is `undefined` — not for other falsy values like `0`, `""`, or `null`:

```javascript
function show(value = "default") {
  console.log(value);
}

show(0);     // 0 (not "default")
show("");    // "" (not "default")
show(null);  // null (not "default")
show();      // "default"
```

Before ES2015, developers used patterns like `name = name || "World"` inside the function body. Default parameters are cleaner and handle edge cases correctly.

## Arrow Functions

Arrow functions are a concise syntax introduced in ES2015. They are written with `=>` instead of the `function` keyword:

```javascript
// Traditional function
function double(x) {
  return x * 2;
}

// Arrow function equivalent
const double = (x) => x * 2;
```

When the body is a single expression, the return is implicit — no braces or `return` keyword needed:

```javascript
const square = (x) => x * 2;
const add = (a, b) => a + b;
const hello = () => "Hello!";
```

For multiple statements, use curly braces and an explicit `return`:

```javascript
const classify = (n) => {
  if (n > 0) {
    return "positive";
  } else if (n < 0) {
    return "negative";
  }
  return "zero";
};
```

Arrow functions are stored in variables (`const` or `let`) since they have no name of their own. Unlike function declarations, they are **not** hoisted — you must define them before calling them.

One important difference: arrow functions do not have their own `this` binding. This matters when working with objects and classes, which you will encounter later. For now, both forms work the same way for standalone functions.

## Scope

Scope determines where a variable is accessible. JavaScript has two main kinds of scope:

**Block scope** (with `let` and `const`): Variables are only accessible inside the block `{}` where they are declared:

```javascript
if (true) {
  let x = 10;
  console.log(x); // 10
}
// console.log(x); // Error! x is not defined here
```

**Function scope**: Variables declared inside a function are not accessible outside it:

```javascript
function test() {
  let secret = 42;
  console.log(secret); // 42
}
test();
// console.log(secret); // Error! secret is not defined here
```

Inner functions can read variables from outer scopes:

```javascript
let greeting = "Hello";

function greet(name) {
  console.log(greeting + ", " + name); // can access outer 'greeting'
}

greet("Alice"); // Hello, Alice
```

**Variable shadowing** occurs when an inner scope declares a variable with the same name as an outer one:

```javascript
let x = "global";

function test() {
  let x = "local"; // shadows the outer x
  console.log(x);  // "local"
}

test();
console.log(x); // "global" (unchanged)
```

The two `x` variables are completely independent. The inner one does not affect the outer one.

**Closures** are a related concept — when a function "remembers" variables from the scope where it was created, even after that scope has finished executing. You will encounter closures more as you advance.

## Recursion

A recursive function is one that calls itself. Every recursive function needs two parts:

1. **Base case**: A condition that stops the recursion.
2. **Recursive case**: The function calls itself with a "smaller" problem.

The classic example is factorial (n! = n * (n-1) * ... * 1):

```javascript
function factorial(n) {
  if (n === 0) {     // base case
    return 1;
  }
  return n * factorial(n - 1); // recursive case
}

console.log(factorial(5)); // 120
```

The call chain builds up a "call stack":

```
factorial(5)
  → 5 * factorial(4)
    → 4 * factorial(3)
      → 3 * factorial(2)
        → 2 * factorial(1)
          → 1 * factorial(0)
            → 1 (base case!)
```

Then the results multiply back: 1 * 1 * 2 * 3 * 4 * 5 = 120.

Without a base case, the function would call itself forever until JavaScript throws a "Maximum call stack size exceeded" error.

Recursion is useful for problems that naturally break into smaller subproblems — like traversing trees, exploring nested structures, or mathematical sequences. For simple counting loops, a regular `for` or `while` loop is usually simpler.

## Putting It All Together

Here is an example that combines several concepts from this lesson:

```javascript
const operations = {
  add: (a, b) => a + b,
  subtract: (a, b) => a - b,
  multiply: (a, b) => a * b,
};

function calculate(op, a, b) {
  const fn = operations[op];
  if (!fn) {
    return "Unknown operation";
  }
  return fn(a, b);
}

console.log(calculate("add", 10, 5));      // 15
console.log(calculate("multiply", 4, 3));  // 12
console.log(calculate("subtract", 20, 8)); // 12
```

This combines:
- **Arrow functions** stored as values in an object.
- **A regular function** (`calculate`) that looks up and calls the right operation.
- **Return values** flowing from the arrow function through `calculate` back to the caller.
- **Parameters** and **scope** working together — `fn` is local to `calculate`, while `operations` is accessible from the outer scope.

Functions are the building blocks you will use for everything going forward. The exercises that follow will help you practice each concept individually.
