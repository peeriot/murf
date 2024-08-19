`murf` is a **M**ocking and **U**nit test **F**ramework for **R**ust, that was inspired by the [gmock](http://google.github.io/googletest/gmock_cook_book.html) framework implemented by google.

`murf` is currently development and heavily used by internal projects of the [peeriot](https://github.com/peeriot) organization. We thought that `murf` may be useful for other developers as well, so we decided to make it open source. We think it is helpful for RUST developers.

If the solution appeals to you we would of course be very happy about your contribution to the project.

We also appreciate any feedback on the solution. It helps us to improve the crate.

We are looking forward to your feedback :)


## Features

`murf` has a wide list of features. To get a better overview here are the most important once:

- `murf` uses proc marcos to generate a mocked versions of your traits and types. This makes it very easy to use.
- `murf` is only a dev-dependency. Which keeps your productive code clean.
- `murf` uses `Matcher` (used to check the arguments of an expected function call) and `Action` (action that is executed for an expected function call) traits (as known from gmock) you can use to implement custom behaviour. This makes it very easy to extend.
- `murf` uses a handle which you can use to add more expectations while the actual mock object was already passed to the code under test. This makes is more flexible.
- `murf` is able to deal with local references as function arguments as well as return values.
- `murf` is able to handle different `self` arguments and return types (like `&Self`, `&mut Self`, `Box<Self>`, `Pin<&mut Self>`, `Arc<Self>` and more)
- `murf` supports generic traits and associated types
- `murf` supports default actions for the mocked methods
- `murf` is able to handle expectations in a defined sequence
- `murf` supports checkpoints to validate all expectations at a given point
- `murf` is able to handle a call count for a defined expectation (with support for ranges)
- `murf` supports mocking associated functions as well (so you can mock constructors like `MyTrait::new()`)


## How to use

The following section will give simple code examples how to use `murf` in your environment to create mocked objects. For more detailed examples please have a look into the `tests` directory. For each feature that is supported by `murf` we have at least one example that shows how to use it.

### Simple example

The following example shows a service that uses a trait to execute some code. This trait is then mocked using `murf` and passed to service instead a real implementation. So the code of the service can be tested against the trait.

```rust
use murf::{mock, expect_method_call, matcher::eq, action::Return};

/// Simple trait that executes something once `exec` is called.
trait MyTrait {
    fn exec(&self, x: usize) -> usize;
}

/// A service that uses [`MyTrait`]
struct Service<T: MyTrait> {
    inner: T,
}

impl<T: MyTrait> Service<T> {
    fn new(inner: T) -> Self {
        Self { inner }
    }

    fn exec(&self) -> usize {
        self.inner.exec(4)
    }
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl MyTrait for MyStruct {
        fn exec(&self, x: usize) -> usize;
    }
}

fn main() {
    let mock = MyStruct::mock();

    expect_method_call!(mock as MyTrait, exec(eq(4))).will_once(Return(104));

    let service = Service::new(mock);

    assert_eq!(104, service.exec());
}
```

### Using handles

Instead of defining all expectations before the mocked object is passed to the code under test, you can use a so called handle to control and manipulate the mocked object.

```rust
use murf::{mock, expect_method_call, matcher::eq, action::Return};

/// Simple trait that executes something once `exec` is called.
trait MyTrait {
    fn exec(&self, x: usize) -> usize;
}

/// A service that uses [`MyTrait`]
struct Service<T: MyTrait> {
    inner: T,
}

impl<T: MyTrait> Service<T> {
    fn new(inner: T) -> Self {
        Self { inner }
    }

    fn exec(&self) -> usize {
        self.inner.exec(4)
    }
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl MyTrait for MyStruct {
        fn exec(&self, x: usize) -> usize;
    }
}

fn main() {
    let (handle, mock) = MyStruct::mock_with_handle();

    // Move the mocked object to the service
    let service = Service::new(mock);

    // Use the handle to control the mocked object
    expect_method_call!(handle as MyTrait, exec(eq(4))).will_once(Return(104));
    expect_method_call!(handle as MyTrait, exec(eq(4))).will_once(Return(105));

    assert_eq!(104, service.exec());
    assert_eq!(105, service.exec());

    handle.checkpoint();

    expect_method_call!(handle as MyTrait, exec(eq(4))).will_once(Return(106));

    assert_eq!(106, service.exec());
}
```

### Using sequences

By default expectations are not bound to a specific order. As long as all defined expectations are executes with the correct parameters, once the handle is dropped, no error is raised. To bind expectations to a specific order you can use [`Sequence`] or [`InSequence`].

```rust
use murf::{mock, expect_method_call, InSequence, action::Return};

/// Simple trait that executes something once `exec` is called.
trait MyTrait {
    fn exec(&self, x: usize) -> usize;
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl MyTrait for MyStruct {
        fn exec(&self, x: usize) -> usize;
    }
}

fn main() {
    let seq = InSequence::default();
    let mock = MyStruct::mock();

    expect_method_call!(mock as MyTrait, exec(_)).will_once(Return(4));
    expect_method_call!(mock as MyTrait, exec(_)).will_once(Return(5));
    expect_method_call!(mock as MyTrait, exec(_)).will_once(Return(6));

    assert_eq!(4, mock.exec(1));
    assert_eq!(5, mock.exec(2));
    assert_eq!(6, mock.exec(3));
}
```

### Using call counts

From time to time it might be also interesting to restrict the expected call count for an expectation. This can be done by using the `times` method of the expectation builder.

If you use `times` in combination with [`Sequence`] the number of calls to an expectation needs to match the expected call count before the next expectation in the sequence is considered active.

```rust
use murf::{mock, expect_method_call, InSequence};
use murf::matcher::eq;
use murf::action::Return;

/// Simple trait that executes something once `exec` is called.
trait MyTrait {
    fn exec(&self, x: usize) -> usize;
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl MyTrait for MyStruct {
        fn exec(&self, x: usize) -> usize;
    }
}

fn main() {
    let seq = InSequence::default();
    let mock = MyStruct::mock();

    expect_method_call!(mock as MyTrait, exec(eq(1)))
        .times(..2) // 0-1 times
        .will_repeatedly(Return(4));
    expect_method_call!(mock as MyTrait, exec(eq(2)))
        .times(1..) // at least one time
        .will_repeatedly(Return(5));
    expect_method_call!(mock as MyTrait, exec(eq(3)))
        .times(2..) // at least two times
        .will_repeatedly(Return(6));

    assert_eq!(5, mock.exec(2));
    assert_eq!(6, mock.exec(3));
    assert_eq!(6, mock.exec(3));
}
```

### Using matchers

To specify what arguments are expected for a call you can use so called [`Matcher`]s. If you are not interested in verifying a certain argument you can use the [`any`](crate::matcher::any) matcher or simply a `_` in the `expect_method_call!` macro.

```rust
use murf::{mock, expect_method_call};
use murf::matcher::{str_starts_with, eq};
use murf::action::Return;

/// Simple trait that executes something once `exec` is called.
trait MyTrait {
    fn exec(&self, a: usize, b: &str, c: usize);
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl MyTrait for MyStruct {
        fn exec(&self, a: usize, b: &str, c: usize);
    }
}

fn main() {
    let mock = MyStruct::mock();

    expect_method_call!(mock as MyTrait, exec(eq(1), str_starts_with("Hello"), _));

    mock.exec(1, "Hello World :)", 1234);
}
```

### Nesting matchers

Matchers can also be nested. This is useful for example if you want to manipulate an argument before it is passed to the actual matcher.

```rust
use std::ops::Deref;

use murf::{mock, expect_method_call};
use murf::matcher::{deref, eq};
use murf::action::Return;

/// Simple trait that executes something once `exec` is called.
trait MyTrait {
    fn exec(&self, a: Wrapper);
}

struct Wrapper(usize);

impl Deref for Wrapper {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl MyTrait for MyStruct {
        fn exec(&self, a: Wrapper);
    }
}

fn main() {
    let mock = MyStruct::mock();

    // Using `eq` directly would cause an error.
    // expect_method_call!(mock as MyTrait, exec(eq(1)));

    expect_method_call!(mock as MyTrait, exec(deref(eq(1))));

    mock.exec(Wrapper(1));
}
```

## Comparison to other crates

`murf` is not the only mocking and unit test framework out there, but murf is the only one that combines the best features of all other crates.

- `v` fully supported
- `-` partially supported
- `x` not supported
- `?` unknown

| Feature | `murf` | `mockall` | `mockers` | `mock_derive` | `galvanic_mock` | `pseudo` | `faux` | `unimock` | `mry` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| maintained | v | v | - | x | x | x | - | v | - |
| documentation | v | v | v | - | - | - | v | v | - |
| proc macro | v | v | v | v | v | x | v | v | v |
| dev-dependency only | v |v | - | - | - | v | - | - | - |
| matcher / action interface | v | v | x | x | - | - | v | v | - |
| split into handle and mock object | v | x | v | x | x | x | x | x | x |
| support for local references | v | v | x | ? | ? | x | - | x | ? |
| support for `Self` type | v | v | v | ? | ? | v | ? | v | ? |
| support for generic traits | v | v | v | v | v | - | x | - |- |
| support for associated types | v |v | - | ? | - | - | x | - | - |
| support for associated functions | v | v | x | ? | x | - | x | x | v |
| support for default actions | v | x | x | v | x | - | x | x | v |
| define expectations in a sequence | v | v | v | ? | x | x | x | - | x |
| support for checkpoints | v | v | v | ? | x | - | x | x | x |
| define call count for expectations | v | v | v | x | v | - | v | v | v |

## License

This project is licensed under the [MIT license](https://choosealicense.com/licenses/mit/).
