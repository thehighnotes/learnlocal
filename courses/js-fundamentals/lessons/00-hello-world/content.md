# Hello World & Getting Started

JavaScript is one of the most widely used programming languages in the world. It
runs in every web browser, powers servers through Node.js, builds desktop apps,
mobile apps, and command-line tools. In this course, you will learn JavaScript
fundamentals by running programs in the terminal with Node.js — no browser
needed.

## Your First Program

Every programming journey starts with "Hello, World!" — a program that prints a
message to the screen. In JavaScript, it looks like this:

```javascript
console.log("Hello, World!");
```

Save that line in a file called `hello.js`, then run it in your terminal:

```
node hello.js
```

You should see `Hello, World!` printed to the screen. That is it — one line of
code, one command to run it. No compilation step, no boilerplate, no `main()`
function. Node.js reads your file from top to bottom and executes each statement.

## The console.log() Function

`console.log()` is the primary way to display output in JavaScript. It prints
its arguments to the terminal, followed by a newline.

You can pass it a string:

```javascript
console.log("Hello, World!");
```

A number:

```javascript
console.log(42);
```

Or multiple arguments separated by commas — they will be printed with spaces
between them:

```javascript
console.log("The answer is", 42);
// Output: The answer is 42
```

Each `console.log()` call prints on its own line:

```javascript
console.log("Line one");
console.log("Line two");
console.log("Line three");
```

This produces three lines of output. The newline is added automatically — you do
not need to add one yourself.

## Strings in JavaScript

A **string** is a sequence of characters — text. JavaScript gives you three ways
to write string literals:

**Double quotes:**

```javascript
console.log("Hello, World!");
```

**Single quotes:**

```javascript
console.log('Hello, World!');
```

These two are identical in behavior. Use whichever you prefer, but be consistent
within a project. Many JavaScript style guides prefer single quotes.

**Backticks (template literals):**

```javascript
const name = "Alice";
console.log(`Hello, ${name}!`);
// Output: Hello, Alice!
```

Backtick strings have a superpower: you can embed expressions inside `${...}`.
The expression is evaluated and its result is inserted into the string. This is
called **string interpolation** and it is one of the most useful features in
JavaScript.

Backtick strings can also span multiple lines:

```javascript
console.log(`Line one
Line two
Line three`);
```

Regular quoted strings cannot span multiple lines without escape characters.

### Comments

Comments are notes in your code that JavaScript ignores. They exist for humans
reading the code — including your future self.

**Single-line comments** start with `//`. Everything after `//` on that line is
ignored:

```javascript
// This is a comment
console.log("Hello"); // This prints Hello
```

**Multi-line comments** are enclosed in `/* */`:

```javascript
/*
  This is a multi-line comment.
  It can span as many lines as you want.
*/
console.log("Hello");
```

Use comments to explain *why* your code does something, not *what* it does. The
code itself shows the "what" — comments should provide context that is not
obvious from reading the code.

## Escape Characters

Sometimes you need to include special characters in a string that you cannot type
directly. **Escape sequences** start with a backslash (`\`) and represent these
special characters:

| Escape | Meaning              | Example Output     |
|--------|----------------------|--------------------|
| `\n`   | Newline              | starts a new line  |
| `\t`   | Tab                  | horizontal indent  |
| `\\`   | Literal backslash    | `\`                |
| `\"`   | Double quote         | `"`                |
| `\'`   | Single quote         | `'`                |

Using them in code:

```javascript
console.log("First line\nSecond line");
// Output:
// First line
// Second line

console.log("Name\tAge");
console.log("Alice\t30");
// Output:
// Name    Age
// Alice   30

console.log("She said \"hello\"");
// Output: She said "hello"
```

The backslash tells JavaScript "the next character is special, not literal."
Without it, a quote inside a quoted string would end the string prematurely.

## Reading Input

Most programs need input as well as output. In Node.js, reading from the
terminal (standard input) uses the built-in `fs` module:

```javascript
const input = require("fs").readFileSync("/dev/stdin", "utf8").trim();
```

This looks dense, so let us break it down:

- `require("fs")` — loads Node's built-in file system module.
- `.readFileSync("/dev/stdin", "utf8")` — reads all of standard input as a
  UTF-8 string. This **blocks** (waits) until input is complete.
- `.trim()` — removes any trailing newline or whitespace from the input.

Why this approach? Node.js is designed around asynchronous, non-blocking I/O.
That is great for servers, but it complicates simple read-a-line programs. The
synchronous `readFileSync` approach gives us straightforward, top-to-bottom
execution that is easy to reason about while learning. We will cover the
asynchronous approach later.

Here is a complete example that reads a name and prints a greeting:

```javascript
const name = require("fs").readFileSync("/dev/stdin", "utf8").trim();
console.log(`Hello, ${name}!`);
```

If you type `Alice` and press Enter, the output is `Hello, Alice!`.

Notice the template literal (backtick string) with `${name}` — this is where
string interpolation shines. Instead of awkward concatenation like
`"Hello, " + name + "!"`, the template literal reads naturally.

## Checking Your Node.js Version

Beyond writing scripts, the `node` command has several useful modes. Before
anything else, you can check which version is installed:

```bash
node --version
```

This prints something like `v20.11.1`. Node.js versions matter because newer
versions support newer JavaScript features.

## Running a Script

The most common way to run JavaScript code is to save it in a `.js` file and
pass it to Node.js:

```bash
node hello.js
```

Node.js reads your file from top to bottom, executing each statement in order.
If there is a syntax error, Node prints an error message showing the line
number and what went wrong.

## Evaluating Expressions

The `-e` flag runs a JavaScript expression directly from the command line:

```bash
node -e "console.log(6 * 7)"
```

This prints `42`. Useful for quick calculations and testing snippets without
creating a file.
