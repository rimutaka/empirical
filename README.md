# Rust's `lazy_static!` usage benchmarks with detailed explanations 

## Intro

[`lazy_static`](https://crates.io/crates/lazy_static) is one of the foundational crates in the Rust ecosystem.
It lets us use static variables without an explicit initialization call.
I used it many times without giving its performance implications much thought. Putting it inside some deeply nested loop got me worried if all that lazy-static magic has some hidden cost.

The [crate's docs](https://docs.rs/lazy_static/latest/lazy_static/index.html) explain the mechanics behind `lazy_static!` macro as:

> The Deref implementation uses a hidden static variable that is guarded by an atomic check on each access.

That sounds innocuous enough, but I still have questions:

1. Is there any noticeable performance cost incurred by the _atomic check on each access_?
2. If `lazy_static` is used in a sub-module, will it be re-initialized on every call to a function from that module?
3. Is it any slower than initializing a variable manually and passing it to other functions as a parameter?

Without understanding the implementation details of `lazy_static` I figured it would be easier to benchmark it than to dig through its [source code](https://github.com/rust-lang-nursery/lazy-static.rs).


## How to run

* benchmarks: `cargo +nightly bench`
* tests: `cargo +nightly test --benches`


## Results

```bash
$ cargo +nightly bench

test bad_rust_local           ... bench:      40,608 ns/iter (+/- 9,239)
test lazy_static_backref      ... bench:          27 ns/iter (+/- 1)
test lazy_static_external_mod ... bench:          27 ns/iter (+/- 0)
test lazy_static_inner        ... bench:          27 ns/iter (+/- 1)
test lazy_static_local        ... bench:          27 ns/iter (+/- 5)
test lazy_static_reinit       ... bench:          26 ns/iter (+/- 1)
test once_cell_lazy           ... bench:          26 ns/iter (+/- 2)
test vanilla_rust_local       ... bench:          27 ns/iter (+/- 0)
```
The results looked pretty neat. The only outlier was a piece of bad code I put in the benches intentionally to set the baseline.

#### __TL;DR:__ `lazy_static!` is fine, but [`once_cell`](https://docs.rs/once_cell/latest/once_cell/) may be better for new projects.


## Benchmarks in detail

### `bad_rust_local()`

This bench does something obviously stupid - it recompiles the regex within the loop.


```rust
b.iter(|| {
    let compiled_regex = regex::Regex::new(LONG_REGEX).unwrap(); // <-- don't place this inside a loop
    let is_match = compiled_regex.is_match(TEST_EMAIL);
    test::black_box(is_match);
});
```

With 40,608 ns/iter it gives us the baseline for the recompilation cost.


### `vanilla_rust_local()`

There was __NO__ noticeable performance cost incurred by the _atomic check on each access_.

`vanilla_rust_local` bench compiled the regex once and took exactly the same 27 ns/iter as the benches using `lazy_static`.
```rust
let compiled_regex = regex::Regex::new(LONG_REGEX).unwrap(); // <-- compiled once only
b.iter(|| {
    let is_match = compiled_regex.is_match(TEST_EMAIL);
    test::black_box(is_match);
});
```

### `lazy_static_external_mod()`, `lazy_static_inner()`, `lazy_static_local()`, `lazy_static_backref()`

These benches relied on `lazy_static` with the only difference in where it was declared:

  * __lazy_static_local:__ at the root level
  * __lazy_static_inner:__ at a sub-module level (same file)
  * __lazy_static_external_mod:__ at a module placed in a separate file
  * __lazy_static_backref:__ at the root level, used in a sub-module

The `lazy_static` declarations were identical in all cases:

```rust
lazy_static! {
    pub(crate) static ref COMPILED_REGEX: regex::Regex = regex::Regex::new(LONG_REGEX).unwrap();
}
```

The placement of `lazy_static! { ... }` declaration made no difference:
1. The static variable was initialized once only
2. All these benches took ~27 ns/iter each.

### `lazy_static_reinit()`

It is possible to initialize the static variable before or after its first use by calling

```rust
lazy_static::initialize(&STATIC_VAR_NAME);
```

There was no additional performance cost for calling `initialize` for the first time or any number of times after that.
This is inline with the documentation that states:

> Takes a shared reference to a lazy static and initializes it if it has not been already.

### `once_cell`

[`once_cell`](https://docs.rs/once_cell/latest/once_cell/) is just as elegant as `lazy_static!` and is [claimed to be faster](https://github.com/async-rs/async-std/issues/406#issuecomment-547286625). It performed on the par with `lazy_static!` in my basic tests.

The static declaration is a single line of code:

```rust
static COMPILED_REGEX_ONCE_CELL: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(LONG_REGEX).unwrap());
```

and the usage of the static variable is exactly the same as with `lazy_static!`:

```rust
    b.iter(|| {
        let is_match = COMPILED_REGEX_ONCE_CELL.is_match(TEST_EMAIL);
        test::black_box(is_match);
    });
```

There is an [RFC to merge `once_cell` into `std::lazy`](https://github.com/rust-lang/rust/issues/74465) making it part of the standard library. It may be a more future-proof choice if you are starting a new project.


## `lazy_static` alternatives that DO NOT work

### Declaring a static variable

```rust
pub(crate) static STATIC_REGEX: regex::Regex = regex::Regex::new(LONG_REGEX).unwrap();
```

> ERROR: calls in statics are limited to constant functions, tuple structs and tuple variants [rustc E0015](https://doc.rust-lang.org/error-index.html#E0015)

### Declaring a const function

```rust
const fn static_regex() -> regex::Regex {
    regex::Regex::new(LONG_REGEX).unwrap()
}
```

> ERROR: calls in constant functions are limited to constant functions, tuple structs and tuple variants [rustc E0015](https://doc.rust-lang.org/error-index.html#E0015)


# `lazy_static` in depth

##### This section goes deep into the source code of `lazy_static` to really understand how it works. 

The demo code in this project is split into several parts:

* __benches__: the source for the benchmarks in this post 
* __examples/expansion_base.rs__: a minimal implementation to get expanded code from `lazy_static!` macro
* __examples/expanded.rs__: the expanded code generated by `lazy_static!` macro from _expansion_base.rs_
* __src/main.rs__: a self-contained implementation based on the expanded code

Your IDE will be unhappy with some parts of the code if you are on _stable_ channel. Switch to/from _nightly_ with these commands to get rid of the IDE warnings:
```bash
rustup default nightly
rustup default stable
```

## Macro code

Running `cargo expand --example expansion_base` outputs the code generated for the crate. You can find the full output in [examples/expanded.rs](examples/expanded.rs) file.

In short, it looks something like this:

```rust
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
extern crate lazy_static;
use lazy_static::lazy_static;
#[allow(missing_copy_implementations)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
struct COMPILED_REGEX {
    __private_field: (),
}
#[doc(hidden)]
static COMPILED_REGEX: COMPILED_REGEX = COMPILED_REGEX {
    __private_field: (),
};
impl ::lazy_static::__Deref for COMPILED_REGEX {
    type Target = regex::Regex;
    fn deref(&self) -> &regex::Regex {
        #[inline(always)]
        fn __static_ref_initialize() -> regex::Regex {
            regex::Regex::new(".*").unwrap()
        }
        #[inline(always)]
        fn __stability() -> &'static regex::Regex {
            static LAZY: ::lazy_static::lazy::Lazy<regex::Regex> = ::lazy_static::lazy::Lazy::INIT;
            LAZY.get(__static_ref_initialize)
        }
        __stability()
    }
}
impl ::lazy_static::LazyStatic for COMPILED_REGEX {
    fn initialize(lazy: &Self) {
        let _ = &**lazy;
    }
}
fn main() {
    let _x = COMPILED_REGEX.is_match("abc");
}
```

The snippet above depends on some functions provided by `lazy_static` and that's where most of the magic happens.

I distilled it to a simpler version that does not have `lazy_static` as a dependency at all. Skim through it and go to the explanations that follow: 

```rust
struct Lazy<T: Sync>(Cell<Option<T>>, Once);
unsafe impl<T: Sync> Sync for Lazy<T> {}

struct CompiledRegex {
    __private_field: (),
}

static COMPILED_REGEX: CompiledRegex = CompiledRegex {
    __private_field: (),
};

impl Deref for CompiledRegex {
    type Target = regex::Regex;
    fn deref(&self) -> &regex::Regex {
        static LAZY: Lazy<regex::Regex> = Lazy(Cell::new(None), Once::new());

        LAZY.1.call_once(|| {
            LAZY.0.set(Some(regex::Regex::new(LONG_REGEX).unwrap()));
        });

        unsafe {
            match *LAZY.0.as_ptr() {
                Some(ref x) => x,
                None => {
                    panic!("attempted to dereference an uninitialized lazy static. This is a bug");
                }
            }
        }
    }
}
```

There are a few key features in the last snippet to pay attention to:

1. `struct Lazy` holds generic `struct CompiledRegex` that is instantiated as `static COMPILED_REGEX` that actually holds the compiled regex.
2. That long chain is unravelled inside `impl Deref for CompiledRegex` to give us the compiled regex as a static variable
3. `std::sync::Once::call_once()` is used to initialize the regex once only
4. `match *LAZY.0.as_ptr()` gets us the initialized regex from deep inside the chain of structs

If the above code is still a bit confusing, look up [src/main.rs](src/main.rs) for the full version with detailed comments.

Running the program with `cargo run` will execute this demo code from [src/main.rs](src/main.rs) using the snippet above for the _lazy-static_ part:

```rust
fn main() {
    println!("Program started");
    println!("{TEST_EMAIL} is valid: {}", COMPILED_REGEX.is_match(TEST_EMAIL));
    println!("{TEST_NOT_EMAIL} is valid: {}", COMPILED_REGEX.is_match(TEST_NOT_EMAIL));
}
```

and produce this output:

```
Program started

Derefencing CompiledRegex

CompiledRegex initialized

name@example.com is valid: true

Derefencing CompiledRegex

Hello world! is valid: false
```

As you can see, `COMPILED_REGEX` is initialized once only and is dereferenced every time `COMPILED_REGEX` variable is used. 

Q.E.D.? :)
