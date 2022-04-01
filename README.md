# type-census

Track the number of extant instances of your types.

```rust
// 1. import these three items:
use type_census::{counter, Instance, Tabulate};

#[derive(Clone)]
pub struct Foo<T> {
    v: T,
    // 2. add a field of type `Instance<Self>`
    _instance: Instance<Self>,
}

impl<T> Foo<T>
{
    pub fn new(v: T) -> Self
    where
        // 3. add a `Self: Tabulate` bound to constructors
        Self: Tabulate,
    {
        Self {
            v,
            // 4. and initialize your `Instance` field like so:
            _instance: Instance::new(),
        }
    }

    pub fn v(self) -> T {
        self.v
    }
}

// 5. finally, implement `Tabulate` like this:
impl<T> Tabulate for Foo<T> {
    counter!();
}

fn main() {
    // you can now query the number of extant instances of `Foo`!
    assert_eq!(Foo::<i8>::instances(), 0);
    assert_eq!(Foo::<u8>::instances(), 0);

    // the same counter is shared for all generic instantiations
    let mut bar: Vec<Foo<i8>> = vec![Foo::new(0i8); 10];

    assert_eq!(Foo::<i8>::instances(), 10);
    assert_eq!(Foo::<u8>::instances(), 10);

    let _baz: Vec<Foo<u8>> = vec![Foo::new(0u8); 5];

    assert_eq!(Foo::<i8>::instances(), 15);
    assert_eq!(Foo::<u8>::instances(), 15);

    let _ = bar.drain(0..5);

    assert_eq!(Foo::<i8>::instances(), 10);
    assert_eq!(Foo::<u8>::instances(), 10);
}
```
