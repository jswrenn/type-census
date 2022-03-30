// 1. import these three items:
use type_census::{Census, Instance, Tabulate};

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
impl<T: 'static> Tabulate for Foo<T> {
    Census!();
}

fn main() {
    use std::iter;

    // you can now query the number of extant instances of `Foo`!
    assert_eq!(Foo::<i8>::instances(), 0);
    assert_eq!(Foo::<u8>::instances(), 0);

    let mut bar: Vec<Foo<i8>> = iter::repeat(Foo::new(0i8)).take(10).collect();

    assert_eq!(Foo::<i8>::instances(), 10);
    assert_eq!(Foo::<u8>::instances(), 0);

    let _baz: Vec<Foo<u8>> = iter::repeat(Foo::new(0u8)).take(5).collect();

    assert_eq!(Foo::<i8>::instances(), 10);
    assert_eq!(Foo::<u8>::instances(), 5);

    let _ = bar.drain(0..5);

    assert_eq!(Foo::<i8>::instances(), 5);
    assert_eq!(Foo::<u8>::instances(), 5);
}
