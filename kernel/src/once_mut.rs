use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Called while spinning (name borrowed from Linux). Can be implemented to call
/// a platform-specific method of lightening CPU load in spinlocks.
#[cfg(all(feature = "asm", any(target_arch = "x86", target_arch = "x86_64")))]
#[inline(always)]
pub fn cpu_relax() {
    // This instruction is meant for usage in spinlock loops
    // (see Intel x86 manual, III, 4.2)
    unsafe { asm!("pause" :::: "volatile"); }
}

#[cfg(any(not(feature = "asm"), not(any(target_arch = "x86", target_arch = "x86_64"))))]
#[inline(always)]
pub fn cpu_relax() {
}

/// A synchronization primitive which can be used to run a one-time global
/// initialization. Unlike its std equivalent, this is generalized so that The
/// closure returns a value and it is stored. Once therefore acts something like
/// 1a future, too.
///
/// # Examples
///
/// ```
/// #![feature(const_fn)]
/// use spin;
///
/// static START: spin::Once<()> = spin::Once::new();
///
/// START.call_once(|| {
///     // run initialization here
/// });
/// ```
pub struct OnceMut<T> {
    state: AtomicUsize,
    data: UnsafeCell<Option<T>>, // TODO remove option and use mem::uninitialized
}

// Same unsafe impls as `std::sync::RwLock`, because this also allows for
// concurrent reads.
unsafe impl<T: Sync + Sync> Sync for OnceMut<T> {}
unsafe impl<T: Sync + Sync> Send for OnceMut<T> {}

// Four states that a Once can be in, encoded into the lower bits of `state` in
// the Once structure.
const INCOMPLETE: usize = 0x0;
const RUNNING: usize = 0x1;
const COMPLETE: usize = 0x2;
const PANICKED: usize = 0x3;

#[cfg(feature = "core_intrinsics")]
#[inline(always)]
fn unreachable() -> ! {
    unsafe { ::core::intrinsics::unreachable() }
}

#[cfg(not(feature = "core_intrinsics"))]
#[inline(always)]
fn unreachable() -> ! {
    unreachable!()
}

impl<T> OnceMut<T> {
    /// Creates a new `Once` value.
    pub const fn new() -> OnceMut<T> {
        OnceMut {
            state: AtomicUsize::new(INCOMPLETE),
            data: UnsafeCell::new(None),
        }
    }

    fn force_get<'a>(&'a self) -> &'a T {
        match unsafe { &*self.data.get() }.as_ref() {
            None    => unreachable(),
            Some(p) => p,
        }
    }

    /// Performs an initialization routine once and only once. The given closure
    /// will be executed if this is the first time `call_once` has been called,
    /// and otherwise the routine will *not* be invoked.
    ///
    /// This method will block the calling thread if another initialization
    /// routine is currently running.
    ///
    /// When this function returns, it is guaranteed that some initialization
    /// has run and completed (it may not be the closure specified). The
    /// returned pointer points to the return value of when of those
    /// initialization closures.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(const_fn)]
    /// use spin;
    ///
    /// static INIT: spin::Once<usize> = spin::Once::new();
    ///
    /// fn get_cached_val() -> usize {
    ///     *INIT.call_once(expensive_computation)
    /// }
    ///
    /// fn expensive_computation() -> usize {
    ///     // ...
    /// # 2
    /// }
    /// ```
    pub fn call_once<'a, F>(&'a self, builder: F) -> &'a T
        where F: FnOnce(&mut T)
    {
        let mut status = self.state.load(Ordering::SeqCst);

        if status == INCOMPLETE {
            status = self.state.compare_and_swap(INCOMPLETE,
                                                 RUNNING,
                                                 Ordering::SeqCst);
            if status == INCOMPLETE { // We init
                // We use a guard (Finish) to catch panics caused by builder
                let mut finish = Finish { state: &self.state, panicked: true };
                unsafe {
                    *self.data.get() = Some(::core::mem::uninitialized());
                    match *self.data.get() {
                        Some(ref mut value) => builder(value),
                        _ => unreachable!(),
                    }
                };
                finish.panicked = false;

                status = COMPLETE;
                self.state.store(status, Ordering::SeqCst);

                // This next line is strictly an optomization
                return self.force_get();
            }
        }

        loop {
            match status {
                INCOMPLETE => unreachable!(),
                RUNNING => { // We spin
                    cpu_relax();
                    status = self.state.load(Ordering::SeqCst)
                },
                PANICKED => panic!("Once has panicked"),
                COMPLETE => return self.force_get(),
                _ => unreachable(),
            }
        }
    }

    /// Returns a pointer iff the `Once` was previously initialized
    pub fn try<'a>(&'a self) -> Option<&'a T> {
        match self.state.load(Ordering::SeqCst) {
            COMPLETE => Some(self.force_get()),
            _        => None,
        }
    }

    /// Like try, but will spin if the `Once` is in the process of being
    /// initialized
    pub fn wait<'a>(&'a self) -> Option<&'a T> {
        loop {
            match self.state.load(Ordering::SeqCst) {
                INCOMPLETE => return None,
                RUNNING    => cpu_relax(), // We spin
                COMPLETE   => return Some(self.force_get()),
                PANICKED   => panic!("Once has panicked"),
                _ => unreachable(),
            }
        }
    }
}

struct Finish<'a> {
    state: &'a AtomicUsize,
    panicked: bool,
}

impl<'a> Drop for Finish<'a> {
    fn drop(&mut self) {
        if self.panicked {
            self.state.store(PANICKED, Ordering::SeqCst);
        }
    }
}
