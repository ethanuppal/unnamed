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

use std::{borrow::Cow, ffi, ptr};

use accessibility_sys::{
    AXUIElementCopyAttributeValue, AXUIElementCreateApplication,
    AXUIElementGetPid, AXUIElementRef, AXUIElementSetAttributeValue,
    AXValueRef, kAXFocusedWindowAttribute, kAXPositionAttribute,
    kAXSizeAttribute, kAXWindowsAttribute, pid_t,
};
use cocoa::{
    appkit::NSRunningApplication,
    base::{id, nil},
    foundation::{NSArray, NSString},
};
use core_foundation_sys::{
    base::{Boolean, kCFAllocatorNull},
    string::{
        CFStringCreateWithBytesNoCopy, CFStringRef, kCFStringEncodingUTF8,
    },
};
use core_graphics::display::{CFIndex, CFTypeRef};
use snafu::ResultExt;

use crate::{
    AXErrorExt, BundleID, UnnamedError,
    layout::AXRect,
    magic,
    memory::{CopyOnWrite, ManageWithRc, Rc, Unique},
};

#[derive(Clone, Copy)]
pub enum AccessibilityElementKey {
    Position,
    Size,
    Windows,
    FocusedWindow,
}

pub fn create_cfstring_from_static_str(
    string: &'static str,
) -> Result<Unique<CFStringRef>, UnnamedError> {
    // SAFETY:
    // - `kCFAllocatorNull` should be initialized by CoreFoundation.
    // - The buffer contains no length or null byte
    // - The string does not need deallocation
    unsafe {
        Unique::new_const(CFStringCreateWithBytesNoCopy(
            ptr::null(),
            string.as_ptr(),
            string.len() as CFIndex,
            kCFStringEncodingUTF8,
            false as Boolean,
            kCFAllocatorNull,
        ))
    }
    .ok_or(UnnamedError::CouldNotCreateCFObject)
}

impl AccessibilityElementKey {
    fn as_cfstring(&self) -> Result<Unique<CFStringRef>, UnnamedError> {
        let string = match self {
            AccessibilityElementKey::Position => kAXPositionAttribute,
            AccessibilityElementKey::Size => kAXSizeAttribute,
            AccessibilityElementKey::Windows => kAXWindowsAttribute,
            AccessibilityElementKey::FocusedWindow => kAXFocusedWindowAttribute,
        };

        create_cfstring_from_static_str(string)
    }
}

pub trait AccessibilityElement {
    /// # Safety
    ///
    /// It is guaranteed that calls to this function will respect the
    /// instructions in [`Rc::get`].
    unsafe fn inner(&self) -> AXUIElementRef;

    /// # Safety
    ///
    /// todo
    unsafe fn set(
        &mut self,
        key: AccessibilityElementKey,
        value: AXValueRef,
    ) -> Result<(), UnnamedError> {
        let key_cfstring = key.as_cfstring().whatever_context(
            "Failed to construct CFString from accessibility key",
        )?;

        // SAFETY: todo
        unsafe {
            AXUIElementSetAttributeValue(
                self.inner(),
                key_cfstring.get(),
                value as CFTypeRef,
            )
        }
        .into_result()
    }

    /// # Safety
    ///
    /// todo
    unsafe fn get(
        &self,
        key: AccessibilityElementKey,
    ) -> Result<Rc<CFTypeRef>, UnnamedError> {
        let key_cfstring = key.as_cfstring().whatever_context(
            "Failed to construct CFString from accessibility key",
        )?;

        let mut result = ptr::null();

        // SAFETY: todo
        unsafe {
            AXUIElementCopyAttributeValue(
                self.inner(),
                key_cfstring.get(),
                &mut result,
            )
        }
        .into_result()?;

        // SAFETY: todo
        unsafe { Rc::new_const(result) }.ok_or(UnnamedError::UnexpectedNull)
    }

    /// Behaves like [`AccessibilityElement::get`] but does not wrap the pointer
    /// in an [`Rc`], so use with caution. The only check it does it for nullity
    /// (in which case it returns `None`).
    ///
    /// # Safety
    ///
    /// todo
    unsafe fn get_raw(
        &self,
        key: AccessibilityElementKey,
    ) -> Result<Option<CFTypeRef>, UnnamedError> {
        let key_cfstring = key.as_cfstring().whatever_context(
            "Failed to construct CFString from accessibility key",
        )?;

        let mut result = ptr::null();

        // SAFETY: todo
        unsafe {
            AXUIElementCopyAttributeValue(
                self.inner(),
                key_cfstring.get(),
                &mut result,
            )
        }
        .into_result()?;

        // SAFETY: todo
        Ok(if result.is_null() { None } else { Some(result) })
    }
}

pub struct App<'a> {
    inner: Rc<AXUIElementRef>,
    pid: pid_t,
    bundle_id: Cow<'a, str>,
}

impl AccessibilityElement for App<'_> {
    unsafe fn inner(&self) -> AXUIElementRef {
        // SAFETY: todo
        unsafe { self.inner.get() }
    }
}

impl<'a> App<'a> {
    /// You can `bundle_id` and extra calls will be done to determine it from
    /// the `app`.
    ///
    /// # Safety
    ///
    /// `app` is an [`NSRunningApplication`].
    pub unsafe fn from_nsapp(
        app: CopyOnWrite<id>,
        bundle_id: impl Into<Option<&'a str>>,
    ) -> Result<Self, UnnamedError> {
        // SAFETY: `app` is an `Rc`.
        let pid = unsafe { app.get().processIdentifier() };

        // SAFETY: todo
        let inner = unsafe { Rc::new_mut(AXUIElementCreateApplication(pid)) }
            .ok_or(UnnamedError::CouldNotCreateCFObject)?;

        let bundle_id = if let Some(bundle_id) = bundle_id.into() {
            bundle_id.into()
        } else {
            // TODO: doesn't this leak?
            // SAFETY: todo
            let bundle_id_nsstring =
                unsafe { NSRunningApplication::bundleIdentifier(app.get()) };

            if bundle_id_nsstring.is_null() {
                return Err(UnnamedError::UnexpectedNull);
            }

            // SAFETY: todo
            let bundle_id_cstr =
                unsafe { NSString::UTF8String(bundle_id_nsstring) };

            // SAFETY: tod
            unsafe { ffi::CStr::from_ptr(bundle_id_cstr) }.to_string_lossy()
        };

        Ok(Self {
            inner,
            pid,
            bundle_id,
        })
    }

    pub fn pid(&self) -> pid_t {
        self.pid
    }

    pub fn bundle_id(&self) -> &Cow<'a, str> {
        &self.bundle_id
    }

    pub fn get_windows(&self) -> Result<Box<[Window]>, UnnamedError> {
        // SAFETY: todo
        let windows = unsafe { self.get(AccessibilityElementKey::Windows) }
            .whatever_context(
                "Failed to get accessibility elements for app windows",
            )?;

        // SAFETY: `windows` is an `NSArray`.
        let count = unsafe { NSArray::count(windows.get() as id) } as usize;

        let mut ax_windows = Vec::with_capacity(count);
        for i in 0..count {
            // SAFETY: `runningApplicationsWithBundleIdentifier` returns an
            // `NSArray`. Each element is managed by the `NSArray`, so we use
            // `as_rc`.
            let ax_window = unsafe {
                (NSArray::objectAtIndex(windows.get() as id, i as u64)
                    as AXUIElementRef)
                    .as_rc()
            }
            .ok_or(UnnamedError::UnexpectedNull)?;

            //let mut pid = 0;
            //// SAFETY: todo
            //unsafe { AXUIElementGetPid(ax_window.get(), &mut pid) }
            //    .into_result()
            //    .whatever_context(format!(
            //        "Could not get {} window PID",
            //        self.bundle_id
            //    ))?;

            // SAFETY: todo
            ax_windows.push(Window {
                inner: CopyOnWrite::Owned(ax_window),
                //pid,
                bundle_id: self.bundle_id.to_string(),
            });
        }

        Ok(ax_windows.into_boxed_slice())
    }

    pub fn focused_window(&self) -> Result<Option<Window>, UnnamedError> {
        let focused_window_opt =
            // SAFETY: todo
            unsafe { self.get_raw(AccessibilityElementKey::FocusedWindow) }
                .whatever_context(
                    "Failed to get optional focused window for the app",
                )?;
        if let Some(focused_window) = focused_window_opt {
            let focused_window_rc = 
            // SAFETY: todo
            unsafe {
                    (focused_window as AXUIElementRef)
                        .into_rc()
            }
                        .expect("`get_raw` will not return `Some` if the pointer inside is null");
            Ok(Some(Window {
                inner: CopyOnWrite::Owned(focused_window_rc),
                bundle_id: self.bundle_id.to_string(),
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct WindowMagicId(u32);

pub struct Window {
    inner: CopyOnWrite<AXUIElementRef>,
    //pid: pid_t,
    bundle_id: String,
}

impl AccessibilityElement for Window {
    unsafe fn inner(&self) -> AXUIElementRef {
        // SAFETY: todo
        unsafe { self.inner.get() }
    }
}

impl Window {
    /// # Safety
    ///
    /// `element` should be valid throughout the course of this function, and
    /// the returned window should not outlive the `element`.
    pub unsafe fn borrow_inner(
        element: AXUIElementRef,
    ) -> Result<Self, UnnamedError> {
        // SAFETY: todo
        if element.is_null() {
            return Err(UnnamedError::UnexpectedNull);
        }

        let mut pid = 0;
        // SAFETY: todo
        unsafe { AXUIElementGetPid(element, &mut pid) }
            .into_result()
            .whatever_context("Could not get window PID")?;

        // TODO: doesn't this leak?
        // SAFETY: todo
        let running_app = unsafe {
            NSRunningApplication::runningApplicationWithProcessIdentifier(
                nil, pid,
            )
        };
        if running_app.is_null() {
            return Err(UnnamedError::UnexpectedNull);
        }

        // TODO: doesn't this leak?
        // SAFETY: todo
        let bundle_id_nsstring =
            unsafe { NSRunningApplication::bundleIdentifier(running_app) };

        if bundle_id_nsstring.is_null() {
            return Err(UnnamedError::UnexpectedNull);
        }

        // SAFETY: todo
        let bundle_id_cstr =
            unsafe { NSString::UTF8String(bundle_id_nsstring) };

        // SAFETY: todo
        let bundle_id = unsafe { ffi::CStr::from_ptr(bundle_id_cstr) }
            .to_string_lossy()
            .to_string();

        Ok(Self {
            inner: CopyOnWrite::Borrowed(element),
            //pid,
            bundle_id,
        })
    }

    pub fn relayout(&mut self, frame: &AXRect) -> Result<(), UnnamedError> {
        // SAFETY: todo
        unsafe {
            self.set(AccessibilityElementKey::Position, frame.origin.get())
        }
        .whatever_context(format!(
            "Failed to set {} position",
            self.bundle_id
        ))?;

        // SAFETY: todo
        unsafe { self.set(AccessibilityElementKey::Size, frame.size.get()) }
            .whatever_context(format!(
                "Failed to set {} size",
                self.bundle_id
            ))?;

        Ok(())
    }

    pub fn bundle_id(&self) -> BundleID {
        BundleID(&self.bundle_id)
    }

    pub fn magic_id(&self) -> Result<WindowMagicId, UnnamedError> {
        let mut id = 0u32;

        // SAFETY: `self.inner` is non-null and `&mut id` is the sole mutable
        // (non-null) reference to `id`.
        unsafe {
            magic::_AXUIElementGetWindow(self.inner.get(), &mut id)
                .into_result()
                .whatever_context("Failed to get magic window ID")?;
        }

        Ok(WindowMagicId(id))
    }
}
