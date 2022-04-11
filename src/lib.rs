//! Track the number of extant instances of your types.
//!
//! ## Example
//! ```
//! // 1. import these two items:
//! use type_census::{Instance, Tabulate};
//!
//! // 2. Derive `Tabulate`
//! #[derive(Clone, Tabulate)]
//! pub struct Foo<T> {
//!     v: T,
//!     // 3. add a field of type `Instance<Self>`
//!     _instance: Instance<Self>,
//! }
//!
//! impl<T> Foo<T> {
//!     pub fn new(v: T) -> Self
//!     where
//!         // 4. add a `Self: Tabulate` bound to constructors
//!         Self: Tabulate,
//!     {
//!         Self {
//!             v,
//!             // 5. and initialize your `Instance` field like so:
//!             _instance: Instance::new(),
//!         }
//!     }
//!
//!     pub fn v(self) -> T {
//!         self.v
//!     }
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
#![deny(missing_docs)]

use num_traits::identities::one;
use std::marker::PhantomData;

pub mod counter;

use counter::Counter;

/// Automatically derive the implementation of [`Tabulate`].
///
/// By default, this uses [`counter::RelaxedCounter`] to count the instances.
/// You can use a different counter type like so:
/// ```
/// // 1. import these two items:
/// use type_census::{Instance, Tabulate};
/// 
/// // 2. Derive `Tabulate`
/// // This will count instances with a `DistributedCounter` with 32 buckets.
/// #[derive(Clone, Tabulate)]
/// #[Tabulate(Counter = "type_census::counter::DistributedCounter<32>")]
/// pub struct Foo<T> {
///     v: T,
///     // 3. add a field of type `Instance<Self>`
///     _instance: Instance<Self>,
/// }
/// ```
pub use type_census_derive::Tabulate;

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
    /// Constructs a new `Instance<T>`, representing the extant lifetime of
    /// an instance of `T`.
    #[inline(always)]
    pub fn new() -> Self {
        T::counter().add_assign(one());
        Instance {
            _tabulated: PhantomData,
        }
    }
}

impl<T> std::fmt::Debug for Instance<T>
where
    T: Tabulate,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>()).finish()
    }
}

impl<T> Default for Instance<T>
where
    T: Tabulate,
{
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for Instance<T>
where
    T: Tabulate,
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
        T::counter().sub_assign(one());
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

impl<T> Eq for Instance<T> where T: Tabulate {}

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
    /// The type of the counter used to track instances of `Self`.
    type Counter: Counter;

    /// Produces a reference to the counter tracking instances of `Self`.
    fn counter() -> &'static Self::Counter;

    /// Produces the number of extant instances of `T`.
    fn instances() -> <Self::Counter as Counter>::Primitive {
        Self::counter().fetch()
    }
}
