# Arrays

Arrays are ordered collections of values and one of the most fundamental data structures in JavaScript. Nearly every program works with lists of things — user names, pixel colors, test scores, shopping cart items — and arrays are how JavaScript represents them. They are flexible, powerful, and come with a rich set of built-in methods.

## Creating Arrays

The most common way to create an array is with square brackets:

```javascript
const numbers = [1, 2, 3, 4, 5];
const words = ["hello", "world"];
const empty = [];
```

JavaScript arrays can hold mixed types (though this is rarely a good idea):

```javascript
const mixed = [1, "two", true, null, { name: "Alice" }];
```

The `.length` property tells you how many elements an array contains:

```javascript
const fruits = ["apple", "banana", "cherry"];
console.log(fruits.length); // 3
```

You can also create arrays from other things using `Array.from()`:

```javascript
const letters = Array.from("hello"); // ["h", "e", "l", "l", "o"]
const range = Array.from({ length: 5 }, (_, i) => i + 1); // [1, 2, 3, 4, 5]
```

## Accessing Elements

Arrays use zero-based indexing. The first element is at index 0:

```javascript
const colors = ["red", "green", "blue"];
console.log(colors[0]); // "red"
console.log(colors[1]); // "green"
console.log(colors[2]); // "blue"
```

To access the last element, use `arr.length - 1`:

```javascript
console.log(colors[colors.length - 1]); // "blue"
```

Modern JavaScript provides `.at()`, which accepts negative indices:

```javascript
console.log(colors.at(-1));  // "blue" (last)
console.log(colors.at(-2));  // "green" (second-to-last)
```

Accessing an index that does not exist returns `undefined` — it does not throw an error:

```javascript
console.log(colors[10]); // undefined
```

## Mutating Methods

These methods modify the original array in place:

```javascript
const arr = [1, 2, 3];

arr.push(4);      // Add to end     → [1, 2, 3, 4]
arr.pop();        // Remove from end → [1, 2, 3]

arr.unshift(0);   // Add to front    → [0, 1, 2, 3]
arr.shift();      // Remove from front → [1, 2, 3]
```

`.splice()` can add, remove, or replace elements at any position:

```javascript
const items = ["a", "b", "c", "d", "e"];

// Remove 2 elements starting at index 1
items.splice(1, 2);           // items is now ["a", "d", "e"]

// Insert "x", "y" at index 1 (remove 0 elements)
items.splice(1, 0, "x", "y"); // items is now ["a", "x", "y", "d", "e"]

// Replace 1 element at index 2
items.splice(2, 1, "z");      // items is now ["a", "x", "z", "d", "e"]
```

All of these methods modify the original array. If you need to keep the original unchanged, make a copy first.

## Non-Mutating Methods

These methods return new values without changing the original array:

```javascript
const arr = [1, 2, 3, 4, 5];

arr.slice(1, 3);    // [2, 3] — extract from index 1 up to (not including) 3
arr.concat([6, 7]); // [1, 2, 3, 4, 5, 6, 7] — join arrays
arr.join(" - ");    // "1 - 2 - 3 - 4 - 5" — convert to string
arr.includes(3);    // true — check if value exists
arr.indexOf(4);     // 3 — find index of value (-1 if not found)
```

A few methods to be careful with:

```javascript
const letters = ["c", "a", "b"];
letters.reverse(); // ["b", "a", "c"] — WARNING: this mutates!
letters.sort();    // ["a", "b", "c"] — WARNING: this also mutates!
```

Despite feeling like they should return new arrays, `.reverse()` and `.sort()` modify the original. Use `.toReversed()` and `.toSorted()` (newer additions) if you need non-mutating versions.

`.flat()` flattens nested arrays:

```javascript
const nested = [[1, 2], [3, 4], [5]];
console.log(nested.flat()); // [1, 2, 3, 4, 5]
```

## Iterating Over Arrays

The `for...of` loop is the cleanest way to iterate:

```javascript
const fruits = ["apple", "banana", "cherry"];
for (const fruit of fruits) {
  console.log(fruit);
}
```

`.forEach()` calls a function for each element:

```javascript
fruits.forEach((fruit, index) => {
  console.log(`${index}: ${fruit}`);
});
```

The traditional `for` loop gives you full control over the index:

```javascript
for (let i = 0; i < fruits.length; i++) {
  console.log(fruits[i]);
}
```

Use `for...of` when you just need the values. Use `.forEach()` when you want the index as well. Use a traditional `for` loop when you need to skip elements, go backwards, or break out early (you cannot `break` from `.forEach()`).

## map and filter

`.map()` transforms each element and returns a new array:

```javascript
const numbers = [1, 2, 3, 4];
const doubled = numbers.map(n => n * 2);
console.log(doubled); // [2, 4, 6, 8]
```

`.filter()` keeps only the elements that pass a test:

```javascript
const numbers = [1, 2, 3, 4, 5, 6];
const evens = numbers.filter(n => n % 2 === 0);
console.log(evens); // [2, 4, 6]
```

Chain them together for powerful data transformations:

```javascript
const scores = [45, 82, 91, 67, 55, 73, 88];

const highScoreLabels = scores
  .filter(s => s >= 70)
  .map(s => `Score: ${s}`);

console.log(highScoreLabels);
// ["Score: 82", "Score: 91", "Score: 73", "Score: 88"]
```

Both `.map()` and `.filter()` leave the original array unchanged. The callback function receives each element (and optionally the index) as arguments.

## Spread and Destructuring

The spread operator `...` expands an array into individual elements:

```javascript
const a = [1, 2, 3];
const b = [4, 5, 6];
const merged = [...a, ...b]; // [1, 2, 3, 4, 5, 6]
const copy = [...a];         // [1, 2, 3] (shallow copy)
```

It also works when calling functions:

```javascript
const numbers = [5, 2, 8, 1, 9];
console.log(Math.max(...numbers)); // 9
```

Destructuring assigns array elements to variables by position:

```javascript
const [first, second, third] = ["a", "b", "c"];
// first = "a", second = "b", third = "c"
```

Skip elements with empty slots:

```javascript
const [x, , z] = [10, 20, 30];
// x = 10, z = 30 (20 is skipped)
```

The rest pattern collects remaining elements:

```javascript
const [head, ...tail] = [1, 2, 3, 4, 5];
// head = 1, tail = [2, 3, 4, 5]
```

A classic use is swapping variables without a temporary:

```javascript
let x = 1;
let y = 2;
[x, y] = [y, x]; // x = 2, y = 1
```

## Putting It All Together

Here is a practical example that combines several array techniques:

```javascript
const transactions = [100, -50, 200, -30, 150, -80, 50];

// Separate deposits and withdrawals
const deposits = transactions.filter(t => t > 0);
const withdrawals = transactions.filter(t => t < 0);

// Calculate totals
const totalDeposits = deposits.reduce((sum, d) => sum + d, 0);
const totalWithdrawals = withdrawals.reduce((sum, w) => sum + w, 0);

// Get the three largest deposits
const topThree = [...deposits].sort((a, b) => b - a).slice(0, 3);

console.log(`Deposits: ${totalDeposits}`);      // Deposits: 500
console.log(`Withdrawals: ${totalWithdrawals}`); // Withdrawals: -160
console.log(`Top 3: ${topThree.join(", ")}`);    // Top 3: 200, 150, 100
```

Notice how `[...deposits]` creates a copy before sorting, so the original `deposits` array is not rearranged. This is a common pattern when you need to sort but also keep the original order.
