//! Shared counters, suitable for quickly tabulating extant types.
//!
//! The default, [`RelaxedCounter`], is suitable in most circumstances.

use crossbeam_utils::CachePadded;
use num_traits::Num;
use std::sync::atomic::{AtomicIsize, Ordering};

/// A type suitable as a shared census counter.
pub trait Counter: 'static {
    /// The primitive type underlying this counter.
    type Primitive: Num;

    /// A fresh instance of this counter holding the value of `0`.
    const ZERO: Self;

    /// Eventually increase the value of this counter by `n`.
    fn add_assign(&self, n: Self::Primitive);

    /// Eventually decrease the value of this counter by `n`.
    fn sub_assign(&self, n: Self::Primitive);

    /// Eventually retrieve the value of this counter.
    fn fetch(&self) -> Self::Primitive;
}

/// An [`AtomicIsize`] padded and aligned to the cache line size to combat
/// [false sharing].
///
/// As a [`Counter`], this type uses [`Ordering::Relaxed`] for
/// [`Counter::add_assign`], [`Counter::sub_assign`] and [`Counter::fetch`].
///
/// [false sharing]: https://en.wikipedia.org/wiki/False_sharing
#[repr(transparent)]
pub struct RelaxedCounter {
    counter: CachePadded<AtomicIsize>,
}

impl Counter for RelaxedCounter {
    type Primitive = isize;
    const ZERO: Self = Self {
        counter: CachePadded::new(AtomicIsize::new(0)),
    };

    #[inline(always)]
    fn add_assign(&self, n: isize) {
        let _ = self.counter.fetch_add(n, Ordering::Relaxed);
    }

    #[inline(always)]
    fn sub_assign(&self, n: isize) {
        let _ = self.counter.fetch_sub(n, Ordering::Relaxed);
    }

    #[inline(always)]
    fn fetch(&self) -> isize {
        self.counter.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod relaxed_counter {
    use super::*;

    #[test]
    fn zero() {
        let counter = RelaxedCounter::ZERO;
        assert_eq!(counter.fetch(), 0);
    }

    #[test]
    fn increment() {
        let counter = RelaxedCounter::ZERO;
        counter.add_assign(1);
        assert_eq!(counter.fetch(), 1);
    }

    #[test]
    fn decrement() {
        let counter = RelaxedCounter::ZERO;
        counter.sub_assign(1);
        assert_eq!(counter.fetch(), -1);
    }
}

/// A counter that minimizes slowdowns from contenation at the cost of increased
/// memory usage.
///
/// Modeled on the ["adaptive multi-counter" described by Travis Downs][multi].
/// Use this counter type only if [`RelaxedCounter`] performs poorly. Then,
/// benchmark the performance of your code with [`DistributedCounter`] with
/// a bucket count of `1`. Increase the number of buckets (up to your
/// available parallelism) until performance is satisfactory. 
///
/// [multi]: https://travisdowns.github.io/blog/2020/07/06/concurrency-costs.html#adaptive-multi-counter
pub struct DistributedCounter<const BUCKETS: usize> {
    counters: [CachePadded<AtomicIsize>; BUCKETS],
}

impl<const BUCKETS: usize> DistributedCounter<BUCKETS> {
    const fn new() -> Self {
        const BUCKET: CachePadded<AtomicIsize> = CachePadded::new(AtomicIsize::new(0));
        Self {
            counters: [BUCKET; BUCKETS],
        }
    }

    fn thread_id() -> usize {
        use std::sync::atomic::AtomicUsize;
        static THREADS: AtomicUsize = AtomicUsize::new(0);
        thread_local! {
            pub static ID: usize = THREADS.fetch_add(1, Ordering::SeqCst);
        }
        ID.try_with(|id| *id).unwrap_or(0)
    }

    #[inline(always)]
    fn try_add_assign(bucket: &AtomicIsize, n: isize) -> Result<isize, isize> {
        let count = bucket.load(Ordering::SeqCst);
        bucket.compare_exchange_weak(
            count,
            count.wrapping_add(n),
            Ordering::SeqCst,
            Ordering::SeqCst,
        )
    }

    #[inline(always)]
    fn add_assign(&self, n: isize) {
        let id = Self::thread_id();
        let mut bucket = id % BUCKETS;
        loop {
            if Self::try_add_assign(&self.counters[bucket], n).is_ok() {
                return;
            } else {
                bucket = bucket.wrapping_add(1) % BUCKETS;
            }
        }
    }
}

impl<const BUCKETS: usize> Counter for DistributedCounter<BUCKETS> {
    type Primitive = isize;
    const ZERO: Self = Self::new();

    fn add_assign(&self, n: isize) {
        self.add_assign(n)
    }

    fn sub_assign(&self, n: isize) {
        self.add_assign(-n)
    }

    fn fetch(&self) -> isize {
        let mut sum = 0isize;
        for counter in &self.counters {
            sum = sum.wrapping_add(counter.load(Ordering::SeqCst));
        }
        sum
    }
}

#[cfg(test)]
mod distributed_counter {
    use super::*;

    #[test]
    fn zero() {
        let counter = DistributedCounter::<1>::ZERO;
        assert_eq!(counter.fetch(), 0);
    }

    #[test]
    fn increment() {
        let counter = DistributedCounter::<1>::ZERO;
        counter.add_assign(1);
        assert_eq!(counter.fetch(), 1);
    }

    #[test]
    fn decrement() {
        let counter = DistributedCounter::<1>::ZERO;
        counter.sub_assign(1);
        assert_eq!(counter.fetch(), -1);
    }
}

/// A [`Counter`] useful for testing.
///
/// This counter uses [`Ordering::SeqCst`] for [`Counter::add_assign`],
/// [`Counter::sub_assign`] and [`Counter::fetch`].
#[repr(transparent)]
pub struct SeqCstCounter {
    counter: CachePadded<AtomicIsize>,
}

impl Counter for SeqCstCounter {
    type Primitive = isize;
    const ZERO: Self = Self {
        counter: CachePadded::new(AtomicIsize::new(0)),
    };

    #[inline(always)]
    fn add_assign(&self, n: isize) {
        let _ = self.counter.fetch_add(n, Ordering::SeqCst);
    }

    #[inline(always)]
    fn sub_assign(&self, n: isize) {
        let _ = self.counter.fetch_sub(n, Ordering::SeqCst);
    }

    #[inline(always)]
    fn fetch(&self) -> isize {
        self.counter.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod seqcst_counter {
    use super::*;

    #[test]
    fn zero() {
        let counter = SeqCstCounter::ZERO;
        assert_eq!(counter.fetch(), 0);
    }

    #[test]
    fn increment() {
        let counter = SeqCstCounter::ZERO;
        counter.add_assign(1);
        assert_eq!(counter.fetch(), 1);
    }

    #[test]
    fn decrement() {
        let counter = SeqCstCounter::ZERO;
        counter.sub_assign(1);
        assert_eq!(counter.fetch(), -1);
    }
}
