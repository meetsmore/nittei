# New to Rust

As Rust is a relatively new language compared to older languages such as C, Java, and even Javascript, it can be useful to have a rough overview of what makes Rust interesting, and how it differs from these languages (with a heavy focus on Typescript).

First, what is Rust ?

Rust is a modern systems programming language focused on **performance**, **safety**, and **concurrency**, without needing a garbage collector. It's often compared to C++, as it's where its advantages can shine the best (safety without sacrificing performance).

While its main domain is systems programming, it's also a very good language for writing backends. Of course, for most backends/APIs, having awesome performance isn't a hard requirement, and is often considered overkill compared to the complexity that is required to achieve it. But, one of the big advantages of Rust compared to other languages (C++, C, ...) is that it can achieve performance while providing a lot of safety mechanisms. This includes the borrow checker, but it also includes the explicit handling of `undefined` values (via `Option<...>`) and of errors (via `Result<...>`).

For backends, Rust is mainly compared to Golang, as both are performant compiled languages that can be used for backends. Rust is more oriented towards safety and performance, at the cost of increased complexity. On the other hand, Golang's main aim is simplicity, which makes it easy to learn, use, and understand. In short, both have their advantages and disadvantages.

## Comparisons with TypeScript

### 🦀 Variables are immutable by default

```ts
let x = 5;
x = 6;
```

```rust
let x = 5;
// x = 6; // ❌ Error: x is immutable
let mut x = 5;
x = 6; // ✅ Use `mut` to make variables mutable
```

---

### 🧱 Strong, static typing (but with great inference)

```ts
let name: string = "Alice";
```

```rust
let name: &str = "Alice"; // or just `let name = "Alice";` — Rust infers types
```

---

### 🚫 No `null` or `undefined` — use `Option`

```ts
function getUser(): User | null {}
```

```rust
fn get_user() -> Option<User> {
    // Some(user) or None
}
```

---

### 🎯 Pattern matching with `match`

```ts
const role = "admin";
switch (role) {
  case "admin":
  // ...
}
```

```rust
match role.as_str() {
    "admin" => { /* ... */ }
    _ => { /* ... */ }
}
```

---

### 🎒 Ownership and borrowing (the big Rust idea)

Rust tracks memory **at compile time** — no GC.

```ts
function takeName(name: string) {}
const myName = "Bob";
takeName(myName); // OK, string is copied
```

```rust
fn take_name(name: String) {}
let my_name = String::from("Bob");
take_name(my_name); // OK, but ownership moved
// my_name is no longer valid here
```

Use references to **borrow** data instead of moving it:

```rust
fn print_name(name: &String) {}
print_name(&my_name); // ✅ Borrowing, no move
```

---

### 🧵 Async is explicit

Rust doesn't have a built-in runtime like Node — you pick one (like `tokio` or `async-std`).

```ts
async function fetchData() {}
```

```rust
async fn fetch_data() {}
```

To run async functions, use `.await` inside an async runtime.

---

### 🛠 Enums with data

Rust enums are more powerful than TS unions.

```ts
type Result<T> = { ok: true; value: T } | { ok: false; error: string };
```

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

---

### 📦 Dependencies use `Cargo.toml`

Like `package.json`, but for Rust.

```toml
[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

---

### 📚 Tooling is great

- `cargo build` — compile
- `cargo run` — run the app
- `cargo test` — run tests
- `cargo fmt` — format code
- `cargo clippy` — linter

---

### ✅ When in doubt, check the compiler

Rust’s compiler is very strict — but its error messages are famously helpful. Trust it!
