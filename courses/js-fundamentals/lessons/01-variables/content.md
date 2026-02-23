# Variables and Types

Every program needs to store and work with data. In JavaScript, you store
data in **variables** — named containers that hold values. Unlike some
languages, JavaScript is **dynamically typed**: you don't declare what kind
of data a variable holds. The type is determined by the value itself, and
can change at any time.

This lesson covers how to declare variables, what types JavaScript has,
and how to convert between them.

## Declaring Variables

JavaScript has three keywords for declaring variables: `let`, `const`, and `var`.

**const** creates a variable that cannot be reassigned:

```javascript
const language = "JavaScript";
const year = 1995;

language = "Python"; // TypeError: Assignment to constant variable
```

**let** creates a variable that can be reassigned:

```javascript
let score = 0;
console.log(score); // 0

score = 10;
console.log(score); // 10

score = score + 5;
console.log(score); // 15
```

**var** is the old way to declare variables. It has confusing scoping rules
(function-scoped instead of block-scoped) and should be avoided in modern
code. You will encounter it in older tutorials and codebases, but always
use `let` or `const` in your own code.

**Rule of thumb:** use `const` by default. Switch to `let` only when you
need to reassign the variable. This makes your code easier to reason about
because readers know a `const` value will never change.

```javascript
const name = "Alice";   // won't change — use const
let count = 0;          // will be incremented — use let
```

## Data Types

JavaScript has several built-in types. The most common are:

**number** — both integers and floating-point values are the same type:

```javascript
console.log(42);      // integer
console.log(3.14);    // float
console.log(-7);      // negative
console.log(0.1 + 0.2); // 0.30000000000000004 (floating-point math!)
```

**string** — text, enclosed in single quotes, double quotes, or backticks:

```javascript
console.log("hello");    // double quotes
console.log('world');    // single quotes
console.log(`backticks`); // template literal (more on this later)
```

**boolean** — either `true` or `false`:

```javascript
console.log(true);
console.log(false);
console.log(3 > 2);   // true
console.log(1 === 2); // false
```

**undefined** — a variable that has been declared but not assigned a value:

```javascript
let x;
console.log(x); // undefined
```

**null** — an intentional "no value" marker. Unlike undefined (which means
"not yet assigned"), null means "deliberately empty":

```javascript
let result = null; // we'll fill this in later
console.log(result); // null
```

**object** — collections of key-value pairs (covered in a later lesson):

```javascript
const person = { name: "Alice", age: 30 };
console.log(person); // { name: 'Alice', age: 30 }
```

## The typeof Operator

The `typeof` operator returns a string describing the type of a value:

```javascript
console.log(typeof 42);        // "number"
console.log(typeof "hello");   // "string"
console.log(typeof true);      // "boolean"
console.log(typeof undefined); // "undefined"
```

This is useful for checking what kind of data you are working with,
especially when input can be unpredictable.

**The null quirk:** `typeof null` returns `"object"` instead of `"null"`.
This is a famous bug from the very first version of JavaScript in 1995. It
was never fixed because too much existing code depends on the behavior. To
check for null, use strict equality instead:

```javascript
console.log(typeof null);    // "object" (bug!)
console.log(null === null);  // true (correct way to check)
```

## Type Coercion

JavaScript automatically converts types in certain situations. This is
called **type coercion**, and it is one of the most confusing aspects of
the language.

The `+` operator does double duty — it adds numbers and concatenates strings.
When one operand is a string, the other is converted to a string:

```javascript
console.log("5" + 3);   // "53" (string concatenation)
console.log("5" + "3"); // "53" (string concatenation)
console.log(5 + 3);     // 8   (numeric addition)
```

But `-`, `*`, and `/` only do math, so they convert strings to numbers:

```javascript
console.log("5" - 3);   // 2 (numeric subtraction)
console.log("6" * 2);   // 12
console.log("10" / 5);  // 2
```

This asymmetry between `+` and the other operators is a classic source of
bugs. Consider what happens with user input:

```javascript
const input = "100"; // from stdin, always a string
console.log(input + 50);  // "10050" — probably not what you wanted!
console.log(input - 50);  // 50 — works, but inconsistent
```

The lesson: always explicitly convert types rather than relying on coercion.

## Template Literals

Template literals use backticks (`` ` ``) instead of quotes and allow you to
embed expressions directly inside the string with `${...}`:

```javascript
const name = "Alice";
const age = 30;

// Template literal — clean and readable
console.log(`${name} is ${age} years old`);

// String concatenation — harder to read
console.log(name + " is " + age + " years old");
```

You can put any expression inside `${...}`, not just variables:

```javascript
console.log(`2 + 2 = ${2 + 2}`);           // "2 + 2 = 4"
console.log(`Type: ${typeof 42}`);          // "Type: number"
console.log(`Upper: ${"hello".toUpperCase()}`); // "Upper: HELLO"
```

Template literals can also span multiple lines without needing `\n`:

```javascript
const message = `Line one
Line two
Line three`;
console.log(message);
// Line one
// Line two
// Line three
```

Template literals are the preferred way to build strings in modern JavaScript.

## Converting Types

Sometimes you need to explicitly convert between types. JavaScript provides
several built-in functions for this:

**String to number:**

```javascript
console.log(Number("42"));       // 42
console.log(Number("3.14"));     // 3.14
console.log(Number("hello"));    // NaN (Not a Number)

console.log(parseInt("42px"));   // 42 (stops at non-digit)
console.log(parseFloat("3.14")); // 3.14
```

**Number to string:**

```javascript
console.log(String(42));       // "42"
console.log((42).toString());  // "42"
```

**To boolean:**

```javascript
console.log(Boolean(1));      // true
console.log(Boolean(0));      // false
console.log(Boolean("hello")); // true
console.log(Boolean(""));     // false
console.log(Boolean(null));   // false
```

**Reading numbers from stdin** is a common pattern. Input is always a
string, so you must convert:

```javascript
const input = require("fs").readFileSync("/dev/stdin", "utf8").trim();
const num = Number(input);
console.log(num * 2);
```

Without the `Number()` call, `input * 2` would still work (because `*`
forces numeric conversion), but `input + 2` would concatenate. Explicit
conversion avoids this trap.

## Putting It All Together

You now know the fundamentals of JavaScript data:

- **Declare** variables with `const` (default) or `let` (when reassignment is needed)
- **Types** include number, string, boolean, undefined, null, and object
- **typeof** tells you what type a value is (watch out for the null quirk)
- **Coercion** automatically converts types — the `+` operator is the main gotcha
- **Template literals** with backticks and `${...}` are the modern way to build strings
- **Explicit conversion** with `Number()`, `String()`, and `parseInt()` is safer than relying on coercion

In the exercises that follow, you will practice each of these concepts.
