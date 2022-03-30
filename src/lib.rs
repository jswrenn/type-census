//! Track the number of extant instances of your types.
//! 
//! ## Example
//! ```rust
//! // 1. import these three items:
//! use type_census::{Census, Instance, Tabulate};
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
//! impl<T: 'static> Tabulate for Foo<T> {
//!     Census!();
//! }
//! 
//! fn main() {
//!     use std::iter;
//! 
//!     // you can now query the number of extant instances of `Foo`!
//!     assert_eq!(Foo::<i8>::instances(), 0);
//!     assert_eq!(Foo::<u8>::instances(), 0);
//! 
//!     let mut bar: Vec<Foo<i8>> = iter::repeat(Foo::new(0i8)).take(10).collect();
//! 
//!     assert_eq!(Foo::<i8>::instances(), 10);
//!     assert_eq!(Foo::<u8>::instances(), 0);
//! 
//!     let _baz: Vec<Foo<u8>> = iter::repeat(Foo::new(0u8)).take(5).collect();
//! 
//!     assert_eq!(Foo::<i8>::instances(), 10);
//!     assert_eq!(Foo::<u8>::instances(), 5);
//! 
//!     let _ = bar.drain(0..5);
//! 
//!     assert_eq!(Foo::<i8>::instances(), 5);
//!     assert_eq!(Foo::<u8>::instances(), 5);
//! }
//! ```

use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::any::Any;
use std::any::TypeId;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Debug, Default)]
struct Hasher;

impl std::hash::BuildHasher for Hasher {
    type Hasher = rustc_hash::FxHasher;

    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        rustc_hash::FxHasher::default()
    }
}

/// A concurrent map from [`TypeId`] to population counts.
pub struct Census {
    counts: Lazy<DashMap<TypeId, &'static AtomicU64, Hasher>>,
}

impl Census {
    pub const EMPTY: Self = Self {
        counts: Lazy::new(|| DashMap::with_hasher(Hasher)),
    };

    fn instances<T: 'static>(&'static self) -> u64 {
        self.counts
            .get(&TypeId::of::<T>())
            .map(|count| count.load(Ordering::SeqCst))
            .unwrap_or(0)
    }
}

/// A guard tracking the lifetime of an instance of `T`.
/// 
/// Constructing an `Instance<T>` increments the population count of `T`.
/// Dropping an `Instance<T>` decrements the population count of `T`. 
pub struct Instance<T> {
    /// The number of instances of `T`
    count: &'static AtomicU64,
    _type: PhantomData<T>,
}

impl<T> Instance<T> {
    /// Construct a new lifetime tracker for an instance of type `T`.
    pub fn new() -> Self
    where
        T: 'static + Tabulate,
    {
        let ty_id = TypeId::of::<T>();
        Instance::from_count(
            T::census()
                .counts
                .entry(ty_id)
                .or_insert_with(|| Box::leak(Box::new(AtomicU64::new(0))))
                .value(),
        )
    }

    #[inline(always)]
    fn from_count(census: &'static AtomicU64) -> Self {
        census.fetch_add(1, Ordering::SeqCst);
        Instance {
            count: census,
            _type: PhantomData,
        }
    }
}

impl<T> Clone for Instance<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self::from_count(self.count)
    }
}

impl<T> Drop for Instance<T> {
    #[inline(always)]
    fn drop(&mut self) {
        self.count.fetch_sub(1, Ordering::SeqCst);
    }
}

impl<T> std::hash::Hash for Instance<T> {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
}

impl<T> Ord for Instance<T> {
    #[inline(always)]
    fn cmp(&self, _: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}

impl<T> PartialOrd for Instance<T> {
    #[inline(always)]
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ordering::Equal)
    }
}

impl<T> Eq for Instance<T> {}

impl<T> PartialEq for Instance<T> {
    #[inline(always)]
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// Track the population of `Self`.
pub trait Tabulate: Any + Sized {
    fn census() -> &'static Census;

    /// Produces the number of extant instances of `Self`.
    fn instances() -> u64
    where
        Self: 'static,
    {
        Self::census().instances::<Self>()
    }
}

/// Generates a correct implementation of [`Tabulate::census`].
///
/// Use like:
/// ```
/// use type_census::{Census, Instance, Tabulate};
/// # pub struct Foo<T> { _v: T }
/// 
/// impl<T: 'static> Tabulate for Foo<T> {
///     Census!();
/// }
/// ```
#[macro_export]
macro_rules! Census {
    () => {
        fn census() -> &'static type_census::Census {
            static CENSUS: type_census::Census = type_census::Census::EMPTY;
            &CENSUS
        }
    };
}
