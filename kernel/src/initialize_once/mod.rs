use ::core::ops::Deref;

use ::core::cell::UnsafeCell;

use ::interrupts::SingleExecution;

enum InitializeData<T> {
    Uninitialized,
    Initialized(T),
}

pub struct InitializeOnce<T> {
    value: UnsafeCell<InitializeData<T>>,
}

unsafe impl<T> Sync for InitializeOnce<T> { }

impl<T> InitializeOnce<T> {
    pub const fn new() -> Self {
        InitializeOnce {
            value: UnsafeCell::new(InitializeData::Uninitialized),
        }
    }

    pub fn initialize(&self, _: &SingleExecution, value: T) {
        unsafe {
            let ptr = self.value.get();
            if let InitializeData::Initialized(_) = *ptr {
                panic!("Attempted to initialize a value twice");
            }
            ::core::ptr::write(ptr, InitializeData::Initialized(value));
        }
    }
}

impl<T> Deref for InitializeOnce<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            match *self.value.get() {
                InitializeData::Uninitialized => panic!("Attempted to access a value before it was initialized"),
                InitializeData::Initialized(ref value) => value,
            }
        }
    }
}
/*
impl<T> DerefMut for InitializeOnce<T> {
    fn deref_mut(&mut self) -> &mut T {
        match self {
            &mut InitializeData::Uninitialized => panic!("Attempted to access a value before it was initialized"),
            &mut InitializeData::Initialized(ref mut value) => value,
        }
    }
}
*/
