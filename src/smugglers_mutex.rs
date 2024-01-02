use std::{
    ops::{Deref, DerefMut},
    sync::{Mutex, MutexGuard},
};

use bevy::ecs::system::Commands;

pub struct SmugglersMutex {
    inner: Mutex<Option<&'static mut Commands<'static, 'static>>>,
}

impl SmugglersMutex {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    pub fn lock(&self) -> SmugglersMutexGuard {
        SmugglersMutexGuard {
            inner: self.inner.lock().unwrap(),
        }
    }

    pub fn share(&self, commands: &mut Commands<'_, '_>, f: impl FnOnce()) {
        let long_life: &'static mut Commands<'static, 'static> =
            unsafe { std::mem::transmute(commands) };

        let mut guard = self.inner.lock().unwrap();
        *guard = Some(long_life);
        drop(guard);

        (f)();

        // Get the mutex back and remove reference before proceeding!
        let mut guard = self.inner.lock().unwrap();
        *guard = None;
    }
}

pub struct SmugglersMutexGuard<'a> {
    inner: MutexGuard<'a, Option<&'static mut Commands<'static, 'static>>>,
}

// impl<'mutex> SmugglersMutexGuard<'mutex> {
//     unsafe fn new(lock: &'mutex SmugglersMutex) -> SmugglersMutexGuard<'mutex> {
//         Self {
//             inner: lock.inner.lock().unwrap(),
//         }
//     }
// }

impl<'a> Deref for SmugglersMutexGuard<'a> {
    type Target = Option<&'a mut Commands<'a, 'a>>;

    fn deref(&self) -> &Self::Target {
        let long_life: &Option<&mut Commands<'_, '_>> = &*self.inner;
        unsafe { std::mem::transmute(long_life) }
    }
}

impl<'a> DerefMut for SmugglersMutexGuard<'a> {
    fn deref_mut(&mut self) -> &mut Option<&'a mut Commands<'a, 'a>> {
        let long_life: &mut Option<&mut Commands<'_, '_>> = &mut *self.inner;
        unsafe { std::mem::transmute(long_life) }
    }
}
