# Operators and Expressions

Operators are symbols that perform operations on values. An **expression** is any combination of values, variables, and operators that produces a result. Every time you write `2 + 3` or `x > 10`, you're writing an expression.

JavaScript has a rich set of operators. In this lesson, you'll learn the ones you'll use every day: arithmetic, comparison, logical, string, assignment, and the ternary operator.

## Arithmetic Operators

JavaScript supports the standard math operations, plus a few extras:

```javascript
console.log(10 + 3);   // 13   — addition
console.log(10 - 3);   // 7    — subtraction
console.log(10 * 3);   // 30   — multiplication
console.log(10 / 3);   // 3.3333...  — division (always floating-point!)
console.log(10 % 3);   // 1    — modulo (remainder)
console.log(2 ** 8);   // 256  — exponentiation (2 to the power of 8)
```

**Important:** JavaScript has no integer division operator. Division always produces a floating-point result. To get integer division, use `Math.floor()`:

```javascript
console.log(10 / 3);            // 3.3333...
console.log(Math.floor(10 / 3)); // 3
```

**Operator precedence** follows standard math rules: `**` first, then `*`, `/`, `%`, and finally `+`, `-`. Use parentheses to make your intent clear:

```javascript
console.log(2 + 3 * 4);     // 14 (multiplication first)
console.log((2 + 3) * 4);   // 20 (parentheses override)
```

## Comparison Operators

Comparison operators compare two values and return a **boolean** (`true` or `false`):

```javascript
console.log(5 > 3);    // true   — greater than
console.log(5 < 3);    // false  — less than
console.log(5 >= 5);   // true   — greater than or equal
console.log(5 <= 4);   // false  — less than or equal
```

These work with strings too. String comparison is **lexicographic** — character by character using Unicode values:

```javascript
console.log("apple" < "banana");  // true  ('a' < 'b')
console.log("cat" < "car");      // false ('t' > 'r')
console.log("a" < "A");          // false (lowercase > uppercase in Unicode)
```

## Strict vs Loose Equality

JavaScript has two kinds of equality checks, and this is one of the most important things to understand early:

```javascript
// Strict equality (===) — no type conversion
console.log(5 === 5);      // true  (same type, same value)
console.log(5 === "5");    // false (number vs string)

// Loose equality (==) — converts types first
console.log(5 == "5");     // true  ("5" converted to 5)
console.log(0 == false);   // true  (false converted to 0)
console.log(0 == "");      // true  ("" converted to 0)
```

Here are some of the most surprising `==` results:

| Expression             | Result | Why                                  |
|------------------------|--------|--------------------------------------|
| `0 == ""`              | true   | Both coerce to 0                     |
| `0 == "0"`             | true   | "0" coerces to 0                     |
| `"" == "0"`            | false  | Both strings, compared as strings    |
| `null == undefined`    | true   | Special rule in the spec             |
| `null == 0`            | false  | null only equals undefined           |
| `false == "false"`     | false  | false→0, "false"→NaN, 0 !== NaN     |
| `false == ""`          | true   | false→0, ""→0                        |

**The rule: always use `===` and `!==`.** The loose equality rules are confusing and lead to bugs. Strict equality does what you expect — no surprises.

```javascript
// Not-equal operators
console.log(5 !== "5");   // true  (strict: different types)
console.log(5 != "5");    // false (loose: same after conversion)
```

## Logical Operators

Logical operators combine or invert boolean values:

```javascript
// && (AND) — true only if BOTH sides are true
console.log(true && true);    // true
console.log(true && false);   // false

// || (OR) — true if EITHER side is true
console.log(false || true);   // true
console.log(false || false);  // false

// ! (NOT) — inverts the value
console.log(!true);           // false
console.log(!false);          // true
```

**Short-circuit evaluation:** JavaScript stops evaluating as soon as the result is determined:

```javascript
// With &&, if the left is false, the right is never evaluated
false && someExpensiveFunction();  // someExpensiveFunction() never runs

// With ||, if the left is true, the right is never evaluated
true || someExpensiveFunction();   // someExpensiveFunction() never runs
```

**Truthy and falsy values:** In a boolean context, these values are **falsy** (treated as false):
- `false`, `0`, `""` (empty string), `null`, `undefined`, `NaN`

Everything else is **truthy** (treated as true), including `"0"`, `[]`, and `{}`.

```javascript
let name = "";
if (!name) {
  console.log("Name is empty");  // This runs — "" is falsy
}
```

## String Operations

The `+` operator concatenates strings when at least one operand is a string:

```javascript
console.log("Hello" + " " + "World");  // "Hello World"
console.log("Score: " + 42);           // "Score: 42" (number converted to string)
```

Useful string methods and properties:

```javascript
let word = "hello";

console.log(word.length);          // 5
console.log(word.toUpperCase());   // "HELLO"
console.log(word.toLowerCase());   // "hello"
console.log("Ha".repeat(3));       // "HaHaHa"
```

Strings are compared lexicographically (dictionary order), character by character:

```javascript
console.log("apple" < "banana");   // true  ('a' comes before 'b')
console.log("abc" < "abd");        // true  (first difference: 'c' < 'd')
console.log("abc" < "ab");         // false (longer string is "greater" when prefix matches)
```

## Assignment Operators

You've already seen `=` for assignment. **Augmented assignment** operators combine an operation with assignment:

```javascript
let x = 10;

x += 5;   // x = x + 5  → 15
x -= 3;   // x = x - 3  → 12
x *= 2;   // x = x * 2  → 24
x /= 4;   // x = x / 4  → 6
x %= 4;   // x = x % 4  → 2
x **= 3;  // x = x ** 3 → 8
```

These are shorter, clearer, and signal intent: "I'm modifying this variable, not computing a new value."

Note: the variable must be declared with `let` (not `const`) since augmented assignment changes the value.

## The Ternary Operator

The **ternary operator** is a compact if/else that fits in a single expression:

```javascript
// Syntax: condition ? valueIfTrue : valueIfFalse

let age = 20;
let status = age >= 18 ? "adult" : "minor";
console.log(status);  // "adult"
```

It's equivalent to:

```javascript
let status;
if (age >= 18) {
  status = "adult";
} else {
  status = "minor";
}
```

Use the ternary operator for simple, one-line choices. For complex logic, use `if/else` — readability matters more than brevity.

You can technically nest ternaries, but don't:

```javascript
// Don't do this — hard to read
let result = x > 0 ? "positive" : x < 0 ? "negative" : "zero";

// Do this instead
if (x > 0) {
  result = "positive";
} else if (x < 0) {
  result = "negative";
} else {
  result = "zero";
}
```

## Putting It All Together

Here's a practical example combining multiple operators:

```javascript
let score = 85;
let bonus = 5;
let maxScore = 100;

// Augmented assignment
score += bonus;  // 90

// Comparison + logical
let passed = score >= 60 && score <= maxScore;  // true

// Ternary
let grade = score >= 90 ? "A" : "B";

// String concatenation
console.log("Score: " + score + "/100 — Grade: " + grade);
// "Score: 90/100 — Grade: A"
```

Operators are the building blocks of every program. You'll combine them constantly as you write conditions, calculations, and transformations. Next up: control flow, where you'll use these operators to make decisions and repeat actions.
