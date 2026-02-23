# Operators and Expressions

Operators are symbols that tell the compiler to perform specific operations on
values. Combined with variables and literals, they form **expressions** -- the
building blocks of any computation.

## Arithmetic Operators

C++ provides the standard math operators you would expect:

| Operator | Meaning        | Example     | Result |
|----------|----------------|-------------|--------|
| `+`      | Addition       | `7 + 3`     | `10`   |
| `-`      | Subtraction    | `7 - 3`     | `4`    |
| `*`      | Multiplication | `7 * 3`     | `21`   |
| `/`      | Division       | `7 / 3`     | `2`    |
| `%`      | Modulus         | `7 % 3`     | `1`    |

**Important:** When both operands are integers, `/` performs **integer division**
-- the result is truncated, not rounded. `7 / 3` gives `2`, not `2.333`.

The modulus operator `%` gives the remainder after division. It only works with
integer types.

```cpp
int a = 17;
int b = 5;
std::cout << a / b << std::endl;  // prints 3 (not 3.4)
std::cout << a % b << std::endl;  // prints 2 (remainder)
```

## Comparison Operators

Comparison operators compare two values and produce a boolean result (`true` or
`false`):

| Operator | Meaning                | Example    | Result  |
|----------|------------------------|------------|---------|
| `==`     | Equal to               | `5 == 5`   | `true`  |
| `!=`     | Not equal to           | `5 != 3`   | `true`  |
| `<`      | Less than              | `3 < 5`    | `true`  |
| `>`      | Greater than           | `3 > 5`    | `false` |
| `<=`     | Less than or equal     | `5 <= 5`   | `true`  |
| `>=`     | Greater than or equal  | `3 >= 5`   | `false` |

A very common mistake is using `=` (assignment) instead of `==` (comparison).
The single `=` assigns a value; the double `==` checks for equality.

```cpp
int x = 10;
if (x == 10) {
    std::cout << "x is ten" << std::endl;
}
```

## Logical Operators

Logical operators combine boolean expressions:

| Operator | Meaning | Example              | Result  |
|----------|---------|----------------------|---------|
| `&&`     | AND     | `true && false`      | `false` |
| `\|\|`  | OR      | `true \|\| false`    | `true`  |
| `!`      | NOT     | `!true`              | `false` |

`&&` returns `true` only when **both** sides are true. `||` returns `true` when
**at least one** side is true. `!` flips a single boolean value.

```cpp
int age = 25;
bool has_license = true;

if (age >= 16 && has_license) {
    std::cout << "Can drive" << std::endl;
}
```

## Operator Precedence

When an expression has multiple operators, C++ follows precedence rules to
decide which operation happens first. The key levels to remember:

1. `!` (NOT) -- highest among these
2. `*`, `/`, `%` -- multiplication, division, modulus
3. `+`, `-` -- addition, subtraction
4. `<`, `<=`, `>`, `>=` -- relational comparisons
5. `==`, `!=` -- equality comparisons
6. `&&` -- logical AND
7. `||` -- logical OR

Multiplication and division happen before addition and subtraction, just like
in standard mathematics:

```cpp
int result = 2 + 3 * 4;   // result is 14, not 20
int forced = (2 + 3) * 4;  // forced is 20 (parentheses override)
```

When in doubt, use parentheses to make your intent clear. Explicit parentheses
also make your code easier to read.

## Compound Assignment Operators

Writing `x = x + 5` is such a common pattern that C++ provides shorthand
operators for it. These **compound assignment** operators apply an operation
and assign the result in one step:

| Operator | Equivalent To    | Example               |
|----------|------------------|-----------------------|
| `+=`     | `x = x + value`  | `score += 10;`       |
| `-=`     | `x = x - value`  | `health -= 25;`      |
| `*=`     | `x = x * value`  | `total *= 2;`        |
| `/=`     | `x = x / value`  | `price /= 4;`        |
| `%=`     | `x = x % value`  | `count %= 10;`       |

These operators do exactly what their expanded forms do -- no more, no less.
They exist purely for convenience and readability.

```cpp
int score = 100;
score += 50;   // score is now 150
score -= 30;   // score is now 120
score *= 2;    // score is now 240
std::cout << score << std::endl;  // prints 240
```

Using compound assignment is idiomatic C++. You will see it far more often
than the longhand `x = x + value` form.

## Increment and Decrement

The `++` and `--` operators add or subtract 1 from a variable. They come in
two flavors: **prefix** and **postfix**.

| Form      | Syntax | Effect                                     |
|-----------|--------|--------------------------------------------|
| Prefix    | `++i`  | Increments `i`, then returns the new value |
| Postfix   | `i++`  | Returns the current value, then increments |

Both forms change the variable by 1. The difference is **what value the
expression produces** when used in a larger statement:

```cpp
int a = 5;
int b = a++;   // b gets 5 (old value), then a becomes 6
int c = ++a;   // a becomes 7 first, then c gets 7
std::cout << a << " " << b << " " << c << std::endl;  // prints 7 5 7
```

When the increment stands alone on its own line (`i++;` or `++i;`), there is
no practical difference -- both just add 1. The distinction only matters when
the result of the expression is used immediately, like in an assignment or a
function argument.

As a rule of thumb: if you only need to bump a value by 1 and don't care about
the expression result, prefer the prefix form `++i`. For integers the
performance is identical, but this habit pays off later with iterators.

## Type Casting with static_cast

Sometimes you need to convert a value from one type to another. C++ provides
`static_cast<type>(value)` for explicit conversions:

```cpp
int a = 7;
int b = 2;
double result = static_cast<double>(a) / b;
std::cout << result << std::endl;  // prints 3.5
```

Without the cast, `a / b` performs integer division and gives `3`. By casting
`a` to `double` first, the compiler promotes `b` to `double` as well, and the
division produces a floating-point result.

Why not just write `7.0 / 2`? When you are working with variables whose values
come from user input, computations, or function returns, you cannot change the
literal -- you need a cast.

`static_cast` is the safe, modern way to convert types in C++. You may see
older C-style casts like `(double)a`, but `static_cast` is preferred because it
is explicit about what conversion is being performed and the compiler can
check that the conversion makes sense.

Common uses of `static_cast`:

- `static_cast<double>(intVar)` -- avoid integer division
- `static_cast<int>(doubleVar)` -- truncate to integer (be aware of data loss)
- `static_cast<char>(intVar)` -- get the ASCII character for a number
