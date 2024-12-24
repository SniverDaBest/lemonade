use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

pub struct Spinlock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> Spinlock<T> {
    pub const fn new(data: T) -> Self {
        Spinlock {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinlockGuard<'_, T> {
        while self.lock.swap(true, Ordering::Acquire) {
            // Spin
        }
        SpinlockGuard { spinlock: self }
    }
}

pub struct SpinlockGuard<'a, T> {
    spinlock: &'a Spinlock<T>,
}

impl<'a, T> Drop for SpinlockGuard<'a, T> {
    fn drop(&mut self) {
        self.spinlock.lock.store(false, Ordering::Release);
    }
}

impl<'a, T> core::ops::Deref for SpinlockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.spinlock.data.get() }
    }
}

impl<'a, T> core::ops::DerefMut for SpinlockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.spinlock.data.get() }
    }
}
