//! Track the number of extant instances of your types.
//! 
//! ## Example
//! ```
//! // 1. import these three items:
//! use type_census::{counter, Instance, Tabulate};
//! 
//! #[derive(Clone)]
//! pub struct Foo<T> {
//!     v: T,
//!     // 2. add a field of type `Instance<Self>`
//!     _instance: Instance<Self>,
//! }
//! 
//! impl<T> Foo<T>
//! {
//!     pub fn new(v: T) -> Self
//!     where
//!         // 3. add a `Self: Tabulate` bound to constructors
//!         Self: Tabulate,
//!     {
//!         Self {
//!             v,
//!             // 4. and initialize your `Instance` field like so:
//!             _instance: Instance::new(),
//!         }
//!     }
//! 
//!     pub fn v(self) -> T {
//!         self.v
//!     }
//! }
//! 
//! // 5. finally, implement `Tabulate` like this:
//! impl<T> Tabulate for Foo<T> {
//!     counter!();
//! }
//! 
//! fn main() {
//!     // you can now query the number of extant instances of `Foo`!
//!     assert_eq!(Foo::<i8>::instances(), 0);
//!     assert_eq!(Foo::<u8>::instances(), 0);
//! 
//!     // the same counter is shared for all generic instantiations
//!     let mut bar: Vec<Foo<i8>> = vec![Foo::new(0i8); 10];
//! 
//!     assert_eq!(Foo::<i8>::instances(), 10);
//!     assert_eq!(Foo::<u8>::instances(), 10);
//! 
//!     let _baz: Vec<Foo<u8>> = vec![Foo::new(0u8); 5];
//! 
//!     assert_eq!(Foo::<i8>::instances(), 15);
//!     assert_eq!(Foo::<u8>::instances(), 15);
//! 
//!     let _ = bar.drain(0..5);
//! 
//!     assert_eq!(Foo::<i8>::instances(), 10);
//!     assert_eq!(Foo::<u8>::instances(), 10);
//! }
//! ```

use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A zero-sized guard that tracks the lifetime of an instance of `T`.
/// 
/// Constructing an `Instance<T>` increments the population count of `T`.
/// Dropping an `Instance<T>` decrements the population count of `T`. 
#[repr(transparent)]
pub struct Instance<T>
where
    T: Tabulate,
{
    _tabulated: PhantomData<T>,
}

impl<T> Instance<T>
where
    T: Tabulate,
{
    #[inline(always)]
    pub fn new() -> Self
    {
        T::counter().fetch_add(1, Ordering::SeqCst);
        Instance {
            _tabulated: PhantomData,
        }
    }
}

impl<T> Clone for Instance<T>
where
    T: Tabulate
{
    #[inline(always)]
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<T> Drop for Instance<T>
where
    T: Tabulate,
{
    #[inline(always)]
    fn drop(&mut self) {
        T::counter().fetch_sub(1, Ordering::SeqCst);
    }
}

impl<T> std::hash::Hash for Instance<T>
where
    T: Tabulate,
{
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
}

impl<T> Ord for Instance<T>
where
    T: Tabulate,
{
    #[inline(always)]
    fn cmp(&self, _: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}

impl<T> PartialOrd for Instance<T>
where
    T: Tabulate,
{
    #[inline(always)]
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ordering::Equal)
    }
}

impl<T> Eq for Instance<T>
where
    T: Tabulate,
{}

impl<T> PartialEq for Instance<T>
where
    T: Tabulate,
{
    #[inline(always)]
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// Track the population of `Self`.
pub trait Tabulate: Sized {
    fn counter() -> &'static AtomicUsize;

    /// Produces the number of extant instances of `Self`.
    fn instances() -> usize
    where
        Self: 'static,
    {
        Self::counter().load(Ordering::SeqCst)
    }
}

/// Generates a correct implementation of [`Tabulate::counter`].
///
/// Use like:
/// ```
/// use type_census::{counter, Instance, Tabulate};
/// # pub struct Foo<T> { _v: T }
/// 
/// impl<T: 'static> Tabulate for Foo<T> {
///     counter!();
/// }
/// ```
#[macro_export]
macro_rules! counter {
    () => {
        fn counter() -> &'static std::sync::atomic::AtomicUsize {
            static COUNTER: std::sync::atomic::AtomicUsize = 
                std::sync::atomic::AtomicUsize::new(0);
            &COUNTER
        }
    };
}
