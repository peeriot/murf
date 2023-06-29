## Murf

`murf` is a **M**ocking and **U**nit test **F**ramework for **R**ust, that was inspired by the [gmock](http://google.github.io/googletest/gmock_cook_book.html) framework implemented by google.

`murf` is currently under development and has no official release. Nevertheless, we believe it is helpful for RUST developers. This is because we have intensively looked around the market for corresponding tools and found that tool support for RUST is still insufficient in this respect.
In the next months we would like to work - as far as our time allows - on the documentation of the tool and also refactor some things.

If the solution appeals to you we would of course be very happy about your contribution to the product.

If you want to know how to use `murf`, please have a look at the [unit tests directory](murf/tests/interface), there we have implemented several small tests for the current feature set of `murf`.

We appreciate any feedback on the solution. It helps us to improve the crate to allow an official release on crates.io (and of course a proper documentation with more examples).

We are looking forward to your feedback :)

## Features

`murf` has a wide list of features. To get a better overview here are the most important once:

- `murf` uses proc marcos to generate the a mocked versions of your traits and types. This makes it very easy to use.
- `murf` is only a dev-dependency. Which keeps your productive code clean.
- `murf` uses `Matcher` (used to check the arguments of an expected function call) and `Action` (action that is executed for an expected function call) traits (as known from gmock) you can use to implement custom behaviour. This makes it very easy to extend.
- `murf` uses a handle which you can use to add more expectations while the actual mock object was already passed to the code under test. This makes is more flexible.
- `murf` is able to deal with local references as function arguments as well as return values.
- `murf` is able to handle different `self` arguments and return types (like `&Self`, `&mut Self`, `Box<Self>`, `Pin<&mut Self>`, `Arc<Self>` and more)
- `murf` supports generic traits and associated types
- `murf` supports default actions for the implemented traits
- `murf` is able to handle expectations in a defined sequence
- `murf` supports checkpoints to validate all expectations at a given point
- `murf` is able to handle a call count for a defined expectation (with support for ranges)
- We plan to add support for associated functions as well (so you can mock constructors like `MyTrait::new()`)

## Example

A simple example how to use `murf`. More detailed examples will be added when we finished the overall documentation. If you need more examples right now, please have a look to the [unit tests directory](murf/tests/interface).

```rust
use murf::{action::Return, expect_call, matcher::eq, mock};

trait MyTrait {
    fn exec(&self, x: usize) -> usize;
}

mock! {
    #[derive(Default)]
    pub struct MyStruct;

    impl MyTrait for MyStruct {
        fn exec(&self, _x: usize) -> usize;
    }
}

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

#[test]
fn success() {
    let (handle, mock) = MyStruct::mock();

    let service = Service::new(mock);

    expect_call!(handle as MyTrait, exec(eq(4))).will_once(Return(4));

    assert_eq!(4, service.exec());
}

#[test]
#[should_panic]
fn failure() {
    let (handle, mock) = MyStruct::mock();

    let service = Service::new(mock);

    expect_call!(handle as MyTrait, exec(_)).will_once(Return(4));

    drop(service);
}
```

## Comparison to other crates

`murf` is not the only mocking and unit test framework out there, but murf is the only one the combines the best features of all other crates.

- :heavy_check_mark: fully supported
- :heavy_plus_sign: partially supported
- :x: not supported
- :black_circle: unknown

| Feature | murf | mockall | mockers | mock_derive | galvanic_mock | pseudo | faux | unimock | mry |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| maintained | :heavy_check_mark: | :heavy_check_mark: | :heavy_plus_sign: | :x: | :x: | :x: | :heavy_plus_sign: | :heavy_check_mark: | :heavy_plus_sign: |
| documentation | :x: (planed) | :heavy_check_mark: | :heavy_check_mark: | :heavy_plus_sign: | :heavy_plus_sign: | :heavy_plus_sign: | :heavy_check_mark: | :heavy_check_mark: | :heavy_plus_sign: |
| proc macro | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :x: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| dev-dependency only | :heavy_check_mark: |:heavy_check_mark: | :heavy_plus_sign: | :heavy_plus_sign: | :heavy_plus_sign: | :heavy_check_mark: | :heavy_plus_sign: | :heavy_plus_sign: | :heavy_plus_sign: |
| matcher / action interface | :heavy_check_mark: | :heavy_check_mark: | :x: | :x: | :heavy_plus_sign: | :heavy_plus_sign: | :heavy_check_mark: | :heavy_check_mark: | :heavy_plus_sign: |
| split into handle and mock object | :heavy_check_mark: | :x: | :heavy_check_mark: | :x: | :x: | :x: | :x: | :x: | :x: |
| support for local references | :heavy_check_mark: | :heavy_check_mark: | :x: | :black_circle: | :black_circle: | :x: | :heavy_plus_sign: | :x: | :black_circle: |
| support for `Self` type | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :black_circle: | :black_circle: | :heavy_check_mark: | :black_circle: | :heavy_check_mark: | :black_circle: |
| support for generic traits | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_plus_sign: | :x: | :heavy_plus_sign: |:heavy_plus_sign: |
| support for associated types | :heavy_check_mark: |:heavy_check_mark: | :heavy_plus_sign: | :black_circle: | :heavy_plus_sign: | :heavy_plus_sign: | :x: | :heavy_plus_sign: | :heavy_plus_sign: |
| support for associated functions | :x: (planed) | :heavy_check_mark: | :x: | :black_circle: | :x: | :heavy_plus_sign: | :x: | :x: | :heavy_check_mark: |
| support for default actions | :heavy_check_mark: | :x: | :x: | :heavy_check_mark: | :x: | :heavy_plus_sign: | :x: | :x: | :heavy_check_mark: |
| define expectations in a sequence | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :black_circle: | :x: | :x: | :x: | :heavy_plus_sign: | :x: |
| support for checkpoints | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :black_circle: | :x: | :heavy_plus_sign: | :x: | :x: | :x: |
| define call count for expectations | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :x: | :heavy_check_mark: | :heavy_plus_sign: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |

## License

This project is licensed under the [MIT license](LICENSE).
