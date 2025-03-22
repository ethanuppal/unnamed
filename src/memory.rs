// Copyright (C) 2024 Ethan Uppal.
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, version 3 of the License only.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along with
// this program.  If not, see <https://www.gnu.org/licenses/>.

//pub struct Manual(pub id);
//
//impl Manual {
//    pub unsafe fn retain(&mut self) {
//        // SAFETY: responsibility of the user
//        unsafe {
//            CFRetain(self.0 as CFTypeRef);
//        }
//    }
//
//    pub unsafe fn release(&mut self) {
//        // SAFETY: responsibility of the user
//        unsafe {
//            CFRelease(self.0 as CFTypeRef);
//        }
//    }
//}

use std::marker::PhantomData;

use cocoa::base::id;
use core_foundation_sys::base::{
    CFGetRetainCount, CFIndex, CFRelease, CFRetain, CFTypeRef,
};

pub struct Rc<T>(
    /// Invariant: if not all `Rc`s have been dropped, then this pointer is
    /// valid. If all `Rc`s referring to the pointer have just been
    /// dropped, then this pointer is invalid.
    CFTypeRef,
    PhantomData<T>,
);

impl<T> Rc<T> {
    /// Gets the number of strong ([`Rc`]) pointers to this allocation.
    pub fn strong_count(&self) -> CFIndex {
        // SAFETY:
        // By the invariant, since we have a reference to a `Rc`, not all `Rc`s
        // referring to the pointer have been dropped, so by the invariant this
        // pointer is valid.
        unsafe { CFGetRetainCount(self.0) }
    }
}

impl<Inner> Rc<*mut Inner> {
    /// Returns `None` if the given pointer is null.
    ///
    /// # Safety
    ///
    /// `pointer` is a valid Apple API object with a nonzero retain count.
    pub unsafe fn new_mut(pointer: *mut Inner) -> Option<Self> {
        if pointer.is_null() {
            None
        } else {
            Some(Self(pointer as CFTypeRef, PhantomData))
        }
    }

    /// # Safety
    ///
    /// You must ensure the returned pointer lives no longer than any `Rc`
    /// whence it comes.
    pub unsafe fn get(&self) -> *mut Inner {
        // SAFETY: By the invariant, since we have a reference to a `Rc`, not
        // all `Rc`s referring to the pointer have been dropped, so by
        // the invariant this pointer is valid. However, we leave the
        // user to responsibly use it from this call.
        self.0 as *mut Inner
    }
}

impl<Inner> Rc<*const Inner> {
    /// Returns `None` if the given pointer is null.
    ///
    /// # Safety
    ///
    /// `pointer` is a valid Apple API object with a nonzero retain count.
    pub unsafe fn new_const(pointer: *const Inner) -> Option<Self> {
        if pointer.is_null() {
            None
        } else {
            Some(Self(pointer as CFTypeRef, PhantomData))
        }
    }

    /// # Safety
    ///
    /// You must ensure the returned pointer lives no longer than any `Rc`
    /// whence it comes.
    pub unsafe fn get(&self) -> *const Inner {
        // SAFETY: By the invariant, since we have a reference to a `Rc`, not
        // all `Rc`s referring to the pointer have been dropped, so by
        // the invariant this pointer is valid. However, we leave the
        // user to responsibly use it from this call.
        self.0 as *const Inner
    }
}

// SAFETY: Only use `<Rc<T> as Clone>` when `T` is a pointer type that can be
// managed by CoreFoundation.
impl<Inner> Clone for Rc<*const Inner> {
    fn clone(&self) -> Self {
        // SAFETY: By the invariant, since we have a reference to a `Rc`, not
        // all `Rc`s referring to the pointer have been dropped, so by
        // the invariant this pointer is valid and we can call
        // `CFRetain` on it.
        Self(unsafe { CFRetain(self.0) }, PhantomData)
    }
}

// SAFETY: Only use `<Rc<T> as Clone>` when `T` is a pointer type that can be
// managed by CoreFoundation.
impl<Inner> Clone for Rc<*mut Inner> {
    fn clone(&self) -> Self {
        // SAFETY: By the invariant, since we have a reference to a `Rc`, not
        // all `Rc`s referring to the pointer have been dropped, so by
        // the invariant this pointer is valid and we can call
        // `CFRetain` on it.
        Self(unsafe { CFRetain(self.0) }, PhantomData)
    }
}

// SAFETY: Only use `<Rc<T> as Drop>` when `T` is a pointer type that can be
// managed by CoreFoundation.
impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        // SAFETY: By the invariant, since we have a reference to a `Rc`, not
        // all `Rc`s referring to the pointer have been dropped, so by
        // the invariant this pointer is valid and we can call
        // `CFRelease` on it.
        unsafe {
            CFRelease(self.0);
        }
    }
}

pub trait ManageWithRc {
    /// Turn an object that you own into an [`Rc`].
    ///
    /// # Safety
    ///
    /// By using this function, you agree to the [`Rc`] invariant.
    unsafe fn into_rc(self) -> Option<Rc<id>>;

    /// Turn an object that is already being memory-managed by another object
    /// into an [`Rc`]. Essentially, this creates a cloned `Rc`.
    ///
    /// # Safety
    ///
    /// By using this function, you agree to the [`Rc`] invariant.
    unsafe fn as_rc(&self) -> Option<Rc<id>>;
}

impl ManageWithRc for id {
    unsafe fn into_rc(self) -> Option<Rc<id>> {
        // SAFETY: user responsibility
        unsafe { Rc::new_mut(self) }
    }

    unsafe fn as_rc(&self) -> Option<Rc<id>> {
        // SAFETY: user responsibility
        let rc = unsafe { Rc::new_mut(*self) }?;

        // SAFETY: `self` is nonnull, but the rest is user responsibility
        unsafe { CFRetain(*self as CFTypeRef) };

        Some(rc)
    }
}
