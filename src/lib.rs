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

use std::{error::Error, marker::PhantomData, ptr};

use accessibility_sys::{
    kAXTrustedCheckOptionPrompt, AXIsProcessTrustedWithOptions,
};
use cocoa::{
    appkit::NSRunningApplication,
    base::{id, nil},
    foundation::{NSArray, NSString},
};
use core_foundation_sys::{
    base::{
        kCFAllocatorDefault, CFGetRetainCount, CFIndex, CFRelease, CFRetain,
        CFTypeRef,
    },
    dictionary::CFDictionaryCreate,
    number::kCFBooleanTrue,
};
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum WiseError {
    #[snafu(display("Failed to create or copy CoreFoundation object"))]
    CouldNotCreateCFObject,
    #[snafu(display("Apple API object was unexpectedly null"))]
    UnexpectedNull,
    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn Error>, Some)))]
        source: Option<Box<dyn Error>>,
    },
}

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

trait ManageWithRc {
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

pub fn has_accessibility_permissions() -> Result<bool, WiseError> {
    // SAFETY: `kAXTrustedCheckOptionPrompt` should be initialized by
    // CoreFoundation.
    let keys = [unsafe { kAXTrustedCheckOptionPrompt } as CFTypeRef];

    // SAFETY: `kCFBooleanTrue` should be initialized by CoreFoundation.
    let values = [unsafe { kCFBooleanTrue } as CFTypeRef];

    // SAFETY:
    // - `keys.as_ptr()` is a valid pointer to a C array of at least 1
    //   pointer-sized value.
    // - `values.as_ptr()` is likewise.
    let options = unsafe {
        Rc::new_const(CFDictionaryCreate(
            kCFAllocatorDefault,
            keys.as_ptr(),
            values.as_ptr(),
            1,
            ptr::null(),
            ptr::null(),
        ))
        .ok_or(WiseError::CouldNotCreateCFObject)
    }?;

    // SAFETY: `options` is a valid dictionary of options.
    let is_trusted = unsafe { AXIsProcessTrustedWithOptions(options.get()) };

    Ok(is_trusted)
}

pub fn running_apps_with_bundle_id(
    bundle_id: &str,
) -> Result<Box<[Rc<id>]>, WiseError> {
    let mut running_apps;

    let bundle_id_nsstring =
    // SAFETY: &str to NSString.
        unsafe { NSString::alloc(nil).init_str(bundle_id).into_rc() }
            .ok_or(WiseError::CouldNotCreateCFObject)?;

    // SAFETY: `bundle_id_nsstring` is nonnull.
    let apps_nsarray = unsafe {
        NSRunningApplication::runningApplicationsWithBundleIdentifier(
            nil,
            bundle_id_nsstring.get(),
        )
        .into_rc()
    }
    .ok_or(WiseError::UnexpectedNull)?;

    // SAFETY: `runningApplicationsWithBundleIdentifier` returns an `NSArray`.
    let count = unsafe { NSArray::count(apps_nsarray.get()) } as usize;

    running_apps = Vec::with_capacity(count);
    for i in 0..count {
        // SAFETY: `runningApplicationsWithBundleIdentifier` returns an
        // `NSArray`. Each element is managed by the `NSArray`, so we use
        // `as_rc`.
        let running_app = unsafe {
            NSArray::objectAtIndex(apps_nsarray.get(), i as u64).as_rc()
        }
        .ok_or(WiseError::UnexpectedNull)?;
        running_apps.push(running_app);
    }

    Ok(running_apps.into_boxed_slice())
}
