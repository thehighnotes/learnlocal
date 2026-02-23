# Objects

Objects are collections of key-value pairs and the most flexible data structure in JavaScript. While arrays store ordered lists accessed by index, objects store named properties accessed by key. Nearly everything in JavaScript is built on objects — they are the foundation of the language.

An object lets you group related data together under meaningful names. Instead of separate variables for a person's name, age, and city, you bundle them into one structure.

## Creating Objects

The most common way to create an object is with an object literal — curly braces containing key-value pairs:

```javascript
const person = {
  name: "Alice",
  age: 30,
  city: "Tokyo"
};
```

Each property has a **key** (also called a name) and a **value**, separated by a colon. Properties are separated by commas.

Keys are strings internally, but you can omit the quotes when the key is a valid identifier (starts with a letter, underscore, or dollar sign, and contains no spaces):

```javascript
// These are equivalent
const a = { name: "Alice" };
const b = { "name": "Alice" };

// Quotes required for keys with spaces or special characters
const c = { "first name": "Alice", "data-type": "user" };
```

Values can be any type — strings, numbers, booleans, arrays, or even other objects:

```javascript
const product = {
  name: "Laptop",
  price: 999.99,
  inStock: true,
  tags: ["electronics", "computers"],
  dimensions: {
    width: 35,
    height: 1.5,
    depth: 24
  }
};
```

An empty object is simply `{}`:

```javascript
const empty = {};
```

## Accessing Properties

There are two ways to access object properties: dot notation and bracket notation.

**Dot notation** is clean and the most common:

```javascript
const person = { name: "Alice", age: 30 };
console.log(person.name);  // "Alice"
console.log(person.age);   // 30
```

**Bracket notation** uses a string (or expression) inside square brackets:

```javascript
console.log(person["name"]);  // "Alice"
console.log(person["age"]);   // 30
```

Bracket notation is required when:

```javascript
// 1. The key is in a variable
const key = "name";
console.log(person[key]);  // "Alice"

// 2. The key has special characters
const obj = { "first name": "Alice" };
console.log(obj["first name"]);  // "Alice"

// 3. The key is computed dynamically
const field = "ag";
console.log(person[field + "e"]);  // 30
```

Access nested properties by chaining:

```javascript
const product = { dimensions: { width: 35 } };
console.log(product.dimensions.width);  // 35
```

Accessing a property that does not exist returns `undefined` (not an error):

```javascript
console.log(person.email);  // undefined
```

## Modifying Objects

You can add, update, and remove properties at any time:

```javascript
const car = { make: "Toyota", color: "red" };

// Add a new property
car.year = 2024;

// Update an existing property
car.color = "blue";

// Delete a property
delete car.color;

console.log(car);  // { make: "Toyota", year: 2024 }
```

Even objects declared with `const` can be modified. The `const` keyword prevents reassigning the variable itself, not changing the object's contents — just like with arrays:

```javascript
const obj = { a: 1 };
obj.b = 2;       // Fine — modifying contents
// obj = { c: 3 };  // Error — reassigning the variable
```

To check if a property exists:

```javascript
const user = { name: "Alice", role: "admin" };

// "in" operator
console.log("name" in user);    // true
console.log("email" in user);   // false

// hasOwnProperty method
console.log(user.hasOwnProperty("role"));   // true
console.log(user.hasOwnProperty("email"));  // false
```

## Object.keys, values, entries

Three static methods convert an object's contents into arrays:

```javascript
const fruit = { apple: 3, banana: 5, cherry: 12 };

// Array of keys
console.log(Object.keys(fruit));
// ["apple", "banana", "cherry"]

// Array of values
console.log(Object.values(fruit));
// [3, 5, 12]

// Array of [key, value] pairs
console.log(Object.entries(fruit));
// [["apple", 3], ["banana", 5], ["cherry", 12]]
```

Since these return arrays, you can use all array methods on the results:

```javascript
const total = Object.values(fruit).reduce((sum, n) => sum + n, 0);
console.log(total);  // 20

const keyCount = Object.keys(fruit).length;
console.log(keyCount);  // 3
```

## Iterating Over Objects

Unlike arrays, you cannot use a regular `for` loop with an index on objects. Here are the main approaches:

**for...in** loops over an object's keys directly:

```javascript
const prices = { tea: 3, coffee: 4.5, juice: 5.5 };

for (const key in prices) {
  console.log(`${key}: ${prices[key]}`);
}
// tea: 3
// coffee: 4.5
// juice: 5.5
```

**Object.entries() with for...of** gives you both key and value via destructuring:

```javascript
for (const [item, price] of Object.entries(prices)) {
  console.log(`${item} costs $${price}`);
}
```

**Object.keys() with forEach**:

```javascript
Object.keys(prices).forEach(key => {
  console.log(`${key}: ${prices[key]}`);
});
```

Each approach works. `Object.entries()` with destructuring is often the cleanest when you need both key and value. `for...in` is the most concise but has a subtlety with inherited properties (not an issue for plain objects).

## Object Destructuring

Destructuring extracts properties into standalone variables:

```javascript
const person = { name: "Alice", age: 30, city: "Tokyo" };

const { name, age, city } = person;
console.log(name);  // "Alice"
console.log(age);   // 30
```

Variable names must match property names. To use a different name, add a colon:

```javascript
const { name: userName, age: userAge } = person;
console.log(userName);  // "Alice"
console.log(userAge);   // 30
```

Set default values for properties that might be missing:

```javascript
const { name, email = "none" } = person;
console.log(email);  // "none" (person has no email property)
```

Destructure nested objects:

```javascript
const order = {
  id: 42,
  customer: { name: "Bob", address: { city: "Paris" } }
};

const { customer: { name, address: { city } } } = order;
console.log(name);  // "Bob"
console.log(city);  // "Paris"
```

Destructuring is especially useful in function parameters:

```javascript
function greet({ name, role }) {
  console.log(`Hello ${name}, you are ${role}`);
}

greet({ name: "Alice", role: "admin", age: 30 });
// Hello Alice, you are admin
```

## Practical Patterns

Objects shine as dictionaries and maps — looking up values by key:

```javascript
const statusMessages = {
  200: "OK",
  404: "Not Found",
  500: "Server Error"
};
console.log(statusMessages[404]);  // "Not Found"
```

**Frequency counter** — counting occurrences:

```javascript
const text = "hello world hello";
const counts = {};
for (const word of text.split(" ")) {
  counts[word] = (counts[word] || 0) + 1;
}
console.log(counts);  // { hello: 2, world: 1 }
```

The pattern `obj[key] = (obj[key] || 0) + 1` is the standard idiom for counting. If the key does not exist yet, `obj[key]` is `undefined`, and `undefined || 0` gives `0`.

**Grouping data**:

```javascript
const people = [
  { name: "Alice", dept: "eng" },
  { name: "Bob", dept: "sales" },
  { name: "Carol", dept: "eng" }
];

const byDept = {};
for (const person of people) {
  if (!byDept[person.dept]) byDept[person.dept] = [];
  byDept[person.dept].push(person.name);
}
console.log(byDept);
// { eng: ["Alice", "Carol"], sales: ["Bob"] }
```

**Configuration objects** — passing named options to functions:

```javascript
function createServer(config) {
  const { port = 3000, host = "localhost", debug = false } = config;
  console.log(`Starting on ${host}:${port}, debug=${debug}`);
}

createServer({ port: 8080, debug: true });
// Starting on localhost:8080, debug=true
```

## Putting It All Together

Here is a practical example that combines creating, accessing, iterating, and destructuring objects:

```javascript
const students = {
  alice: { score: 92, grade: "A" },
  bob: { score: 78, grade: "C" },
  carol: { score: 85, grade: "B" }
};

// Find the top scorer
let topName = "";
let topScore = 0;

for (const [name, { score }] of Object.entries(students)) {
  if (score > topScore) {
    topName = name;
    topScore = score;
  }
}

console.log(`Top student: ${topName} (${topScore})`);
// Top student: alice (92)

// Average score
const scores = Object.values(students).map(s => s.score);
const avg = scores.reduce((sum, s) => sum + s, 0) / scores.length;
console.log(`Average: ${avg.toFixed(1)}`);
// Average: 85.0
```

This demonstrates how objects let you model structured data cleanly and work with it using the methods and patterns covered in this lesson.
