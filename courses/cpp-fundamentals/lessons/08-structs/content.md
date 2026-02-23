# Structs

## Why Structs?

Imagine you are tracking students in a program. With plain variables, you might write:

```cpp
std::string name1 = "Alice";
int age1 = 20;
std::string name2 = "Bob";
int age2 = 22;
```

This gets unwieldy fast. The name and age of each student are related, but the language does not know that. If you add a third field -- say, a GPA -- you now have three separate variables per student, and nothing tying them together.

A **struct** lets you define a custom type that groups related data into a single unit.

## Defining a Struct

Use the `struct` keyword followed by a name and a list of member variables inside braces. By convention, struct names use **PascalCase**.

```cpp
struct Student {
    std::string name;
    int age;
};  // <-- note the semicolon after the closing brace
```

This does not create a variable. It defines a new type called `Student` that you can use just like `int` or `double`.

## Creating Struct Variables

Once defined, you can create variables of that type and access members with the **dot operator** (`.`):

```cpp
Student s;
s.name = "Alice";
s.age = 20;

std::cout << s.name << " is " << s.age << " years old" << std::endl;
```

Each `Student` variable has its own independent copy of `name` and `age`.

## Brace Initialization

You can initialize all members at once using braces. The values are assigned in the order the members are declared:

```cpp
Student s = {"Alice", 20};
```

Or with the more modern uniform initialization syntax:

```cpp
Student s{"Alice", 20};
```

Both create a `Student` with `name` set to `"Alice"` and `age` set to `20`.

## Passing Structs to Functions

Structs can be passed to functions just like any other type. By default, C++ passes by value -- the function gets a copy:

```cpp
void printStudent(Student s) {
    std::cout << s.name << " (" << s.age << ")" << std::endl;
}
```

If the struct is large or you want to avoid copying, pass by **const reference**:

```cpp
void printStudent(const Student& s) {
    std::cout << s.name << " (" << s.age << ")" << std::endl;
}
```

This avoids the copy while preventing the function from modifying the original.

## Returning Structs from Functions

Functions can also return structs. This is a clean way to compute and return multiple related values:

```cpp
struct Point {
    double x;
    double y;
};

Point midpoint(Point a, Point b) {
    return Point{(a.x + b.x) / 2, (a.y + b.y) / 2};
}
```

The caller receives a new `Point` with the computed values.

## Arrays of Structs

You can create arrays of structs to store collections of structured data:

```cpp
Student roster[3] = {
    {"Alice", 20},
    {"Bob", 22},
    {"Charlie", 19}
};

for (int i = 0; i < 3; i++) {
    std::cout << roster[i].name << std::endl;
}
```

This is a very common pattern -- most real programs work with collections of structured records.

## Nested Structs

Structs can contain other structs as members. This lets you model more complex relationships:

```cpp
struct Address {
    std::string city;
    std::string country;
};

struct Person {
    std::string name;
    Address address;
};

Person p = {"Alice", {"Tokyo", "Japan"}};
std::cout << p.name << " lives in " << p.address.city << std::endl;
```

Access nested members by chaining the dot operator: `p.address.city`. Define the inner struct first so the compiler knows about it when it encounters the outer struct.

## Structs vs Classes

In C++, `struct` and `class` are almost identical. The only difference is the default access level:

- **struct**: members are **public** by default
- **class**: members are **private** by default

For simple data grouping without methods or access control, `struct` is the conventional choice. When you start adding member functions, constructors, and private data, most C++ programmers switch to `class`. But technically, anything you can do with one, you can do with the other.

For now, structs are the right tool. You will encounter classes when you study object-oriented programming.

## Dot vs Arrow Operators

When you have a regular struct variable, you use the **dot operator** (`.`) to access members:

```cpp
Student s{"Alice", 20};
std::cout << s.name << std::endl;  // dot operator
```

When you have a **pointer** to a struct, you use the **arrow operator** (`->`):

```cpp
Student* ptr = &s;
std::cout << ptr->name << std::endl;  // arrow operator
```

This is a common source of bugs for beginners -- using `->` on a regular variable or `.` on a pointer. The compiler will tell you if you mix them up.
