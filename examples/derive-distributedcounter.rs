// 1. import these two items:
use type_census::{Instance, Tabulate};

// 2. Derive `Tabulate`
// This will count instances with a `DistributedCounter` with 32 buckets.
#[derive(Clone, Tabulate)]
#[Tabulate(Counter = "type_census::DistributedCounter<32>")]
pub struct Foo<T> {
    v: T,
    // 3. add a field of type `Instance<Self>`
    _instance: Instance<Self>,
}

impl<T> Foo<T> {
    pub fn new(v: T) -> Self
    where
        // 4. add a `Self: Tabulate` bound to constructors
        Self: Tabulate,
    {
        Self {
            v,
            // 5. and initialize your `Instance` field like so:
            _instance: Instance::new(),
        }
    }

    pub fn v(self) -> T {
        self.v
    }
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
