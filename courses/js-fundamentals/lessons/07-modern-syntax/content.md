# Modern JavaScript

ES2015 (also known as ES6) and later versions introduced powerful features that have fundamentally changed how JavaScript is written. These aren't just syntactic sugar — they solve real problems with clarity and safety. This lesson covers the most commonly used modern features that you'll encounter in every JavaScript codebase today.

## Template Literal Expressions

You already know template literals use backticks and `${...}` for interpolation. But the expression inside `${...}` can be *any* valid JavaScript expression — not just variables.

```javascript
const x = 10;
const y = 3;

// Math expressions
console.log(`${x} divided by ${y} is ${x / y}`);
// 10 divided by 3 is 3.3333333333333335

// Function calls
console.log(`Name: ${"alice".toUpperCase()}`);
// Name: ALICE

// Ternary operators
const score = 85;
console.log(`Result: ${score >= 60 ? "pass" : "fail"}`);
// Result: pass
```

Template literals also preserve newlines, making multi-line strings natural:

```javascript
const html = `
<div class="card">
  <h2>${title}</h2>
  <p>${description}</p>
</div>
`;
```

There are also **tagged templates** — where you put a function name before the backtick — but those are an advanced feature you won't need often. Just know they exist for libraries like styled-components and GraphQL queries.

## Rest Parameters

The rest parameter syntax `...` collects remaining arguments into a real array:

```javascript
function sum(...nums) {
  let total = 0;
  for (const n of nums) {
    total += n;
  }
  return total;
}

console.log(sum(1, 2, 3));    // 6
console.log(sum(10, 20));     // 30
console.log(sum(5));          // 5
console.log(sum());           // 0
```

The rest parameter must be the **last** parameter:

```javascript
function log(level, ...messages) {
  for (const msg of messages) {
    console.log(`[${level}] ${msg}`);
  }
}

log("INFO", "Server started", "Listening on port 3000");
// [INFO] Server started
// [INFO] Listening on port 3000
```

Rest parameters replaced the old `arguments` object, which looked like an array but wasn't one — you couldn't use `.map()`, `.filter()`, or `.reduce()` on it without converting it first. Rest parameters give you a proper array from the start.

Practical uses include summing any number of values, creating wrapper functions that forward arguments, and building functions that accept variable-length input.

## Spread with Objects

The spread operator `...` with objects copies properties into a new object:

```javascript
const original = { a: 1, b: 2, c: 3 };
const copy = { ...original };
console.log(copy);  // { a: 1, b: 2, c: 3 }
```

Merging objects — later properties override earlier ones:

```javascript
const defaults = { color: "red", size: "medium" };
const custom = { color: "blue" };
const config = { ...defaults, ...custom };
console.log(config);  // { color: "blue", size: "medium" }
```

Adding or overriding properties inline:

```javascript
const user = { name: "Alice", role: "user" };
const admin = { ...user, role: "admin", verified: true };
console.log(admin);  // { name: "Alice", role: "admin", verified: true }
```

**Shallow copy caveat**: Spread only copies one level deep. Nested objects are still shared references:

```javascript
const original = { name: "Alice", scores: [90, 85] };
const copy = { ...original };
copy.scores.push(100);
console.log(original.scores);  // [90, 85, 100] — original is affected!
```

If you need deep copies, use `structuredClone(obj)` (available in Node.js 17+) or a library.

## Nullish Coalescing (??)

The `??` operator returns the right side only when the left side is `null` or `undefined`:

```javascript
console.log(0 || "default");     // "default" — 0 is falsy
console.log(0 ?? "default");     // 0 — 0 is not null/undefined

console.log("" || "default");    // "default" — "" is falsy
console.log("" ?? "default");    // "" — "" is not null/undefined

console.log(null ?? "default");  // "default" — null triggers ??
console.log(undefined ?? "fb");  // "fb" — undefined triggers ??
```

The difference from `||` matters when `0`, `""`, or `false` are valid values:

```javascript
// Config where 0 is a valid setting
function getTimeout(config) {
  // BAD: || treats 0 as falsy, overrides user's explicit 0
  const bad = config.timeout || 5000;

  // GOOD: ?? only falls back for null/undefined
  const good = config.timeout ?? 5000;

  return { bad, good };
}

console.log(getTimeout({ timeout: 0 }));
// { bad: 5000, good: 0 }
```

Use `??` when you want to provide defaults but preserve intentional falsy values like `0` or `""`. Use `||` when you want to replace *any* falsy value.

## Optional Chaining (?.)

Optional chaining `?.` safely accesses properties on values that might be `null` or `undefined`:

```javascript
const user = {
  name: "Alice",
  address: { city: "Tokyo" }
};

console.log(user.address?.city);    // "Tokyo"
console.log(user.company?.name);    // undefined (no error!)

// Without optional chaining, this would throw:
// console.log(user.company.name);  // TypeError!
```

It short-circuits — if any part is `null` or `undefined`, the whole expression returns `undefined` without evaluating the rest:

```javascript
const data = null;
console.log(data?.deeply?.nested?.value);  // undefined (no error)
```

Optional chaining also works with method calls and bracket notation:

```javascript
const obj = { greet: () => "hello" };
console.log(obj.greet?.());      // "hello"
console.log(obj.missing?.());    // undefined (doesn't call)

const arr = [10, 20, 30];
console.log(arr?.[0]);           // 10
console.log(null?.[0]);          // undefined
```

This is especially useful with API data where nested properties might not exist:

```javascript
const response = { data: { users: [{ name: "Alice" }] } };
const firstName = response?.data?.users?.[0]?.name;
console.log(firstName);  // "Alice"
```

## Property Shorthand and Computed Properties

When a variable name matches the property name you want, use shorthand:

```javascript
const name = "Alice";
const age = 30;

// Longhand
const user1 = { name: name, age: age };

// Shorthand — identical result
const user2 = { name, age };
```

**Computed property names** use brackets to create dynamic keys:

```javascript
const field = "email";
const obj = { [field]: "alice@example.com" };
console.log(obj);  // { email: "alice@example.com" }

// Expressions work too
const prefix = "user";
const data = { [`${prefix}Name`]: "Alice" };
console.log(data);  // { userName: "Alice" }
```

**Method shorthand** lets you define methods without the `function` keyword:

```javascript
const counter = {
  count: 0,
  increment() {
    this.count += 1;
  },
  decrement() {
    this.count -= 1;
  }
};
```

These shorthands are everywhere in modern JavaScript — function returns, module exports, React components, Express routes, and more.

### Putting It All Together

Modern JavaScript features are designed to work together. Here's a practical example combining several features:

```javascript
function createResponse({ status = 200, data = null, message = "OK" }) {
  const timestamp = Date.now();
  return {
    status,             // property shorthand
    message,            // property shorthand
    data: data ?? [],   // nullish coalescing (preserve null? no, default to [])
    timestamp,          // property shorthand
    ok: status < 400    // computed value
  };
}

const res = createResponse({ status: 404, message: "Not Found" });
console.log(`${res.status}: ${res.message}`);  // template literal
console.log(res.data);                          // []
console.log(res.ok);                            // false
```

Each feature removes a small piece of boilerplate. Together, they make JavaScript code significantly more concise and readable. You don't need to use every feature in every function — use what makes the code clearer.

## Command-Line Arguments

Node.js provides command-line arguments through `process.argv`, a global array:

```javascript
// args.js
console.log(process.argv);
```

If you run `node args.js Alice 30`, the array contains:

```javascript
['/usr/bin/node', '/path/to/args.js', 'Alice', '30']
```

- `process.argv[0]` is the path to the node binary
- `process.argv[1]` is the path to your script
- `process.argv[2]` is the first user argument
- `process.argv[3]` is the second user argument, and so on

A practical example:

```javascript
const name = process.argv[2];
console.log(`Hello, ${name}!`);
```

```bash
node greet.js Alice
```

Output: `Hello, Alice!`

Arguments are always strings. Use `Number()` or `parseInt()` to convert when
needed. Command-line arguments let you make scripts configurable without
changing the code.
