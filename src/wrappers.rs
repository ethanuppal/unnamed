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

use std::ptr;

use accessibility_sys::{
    AXUIElementCopyAttributeValue, AXUIElementCreateApplication,
    AXUIElementRef, AXUIElementSetAttributeValue, AXValueRef, kAXErrorSuccess,
    kAXPositionAttribute, kAXSizeAttribute, kAXWindowsAttribute,
};
use cocoa::{appkit::NSRunningApplication, base::id, foundation::NSArray};
use core_foundation_sys::{
    base::{Boolean, kCFAllocatorNull},
    string::{
        CFStringCreateWithBytesNoCopy, CFStringRef, kCFStringEncodingUTF8,
    },
};
use core_graphics::display::{CFIndex, CFTypeRef};
use snafu::ResultExt;

use crate::{
    UnnamedError,
    layout::AXRect,
    memory::{ManageWithRc, Rc, Unique},
};

#[derive(Clone, Copy)]
pub enum AccessibilityElementKey {
    Position,
    Size,
    Windows,
}

impl AccessibilityElementKey {
    fn as_cfstring(&self) -> Result<Unique<CFStringRef>, UnnamedError> {
        let string = match self {
            AccessibilityElementKey::Position => kAXPositionAttribute,
            AccessibilityElementKey::Size => kAXSizeAttribute,
            AccessibilityElementKey::Windows => kAXWindowsAttribute,
        };

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
        let error_code = unsafe {
            AXUIElementSetAttributeValue(
                self.inner(),
                key_cfstring.get(),
                value as CFTypeRef,
            )
        };

        if error_code != kAXErrorSuccess {
            return Err(UnnamedError::AXError { code: error_code });
        }

        Ok(())
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
        let error_code = unsafe {
            AXUIElementCopyAttributeValue(
                self.inner(),
                key_cfstring.get(),
                &mut result,
            )
        };

        if error_code != kAXErrorSuccess {
            return Err(UnnamedError::AXError { code: error_code });
        }

        // SAFETY: todo
        unsafe { Rc::new_const(result) }.ok_or(UnnamedError::UnexpectedNull)
    }
}

pub struct App<'a>(Rc<AXUIElementRef>, &'a str);

impl AccessibilityElement for App<'_> {
    unsafe fn inner(&self) -> AXUIElementRef {
        // SAFETY: todo
        unsafe { self.0.get() }
    }
}

impl<'a> App<'a> {
    /// # Safety
    ///
    /// `app` is an [`NSRunningApplication`].
    pub unsafe fn from_nsapp(
        app: Rc<id>,
        bundle_id: &'a str,
    ) -> Result<Self, UnnamedError> {
        // SAFETY: `app` is an `Rc`.
        let pid = unsafe { app.get().processIdentifier() };

        // SAFETY: todo
        let inner = unsafe { Rc::new_mut(AXUIElementCreateApplication(pid)) }
            .ok_or(UnnamedError::CouldNotCreateCFObject)?;

        Ok(Self(inner, bundle_id))
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

            // SAFETY: todo
            ax_windows.push(Window(ax_window, self.1));
        }

        Ok(ax_windows.into_boxed_slice())
    }
}

pub struct Window<'a>(Rc<AXUIElementRef>, &'a str);

impl AccessibilityElement for Window<'_> {
    unsafe fn inner(&self) -> AXUIElementRef {
        // SAFETY: todo
        unsafe { self.0.get() }
    }
}

impl Window<'_> {
    pub fn resize(&mut self, frame: &AXRect) -> Result<(), UnnamedError> {
        let bundle_id = self.1;

        // SAFETY: todo
        unsafe {
            self.set(AccessibilityElementKey::Position, frame.origin.get())
        }
        .whatever_context(format!("Failed to set {bundle_id} position"))?;

        // SAFETY: todo
        unsafe { self.set(AccessibilityElementKey::Size, frame.size.get()) }
            .whatever_context(format!("Failed to set {bundle_id} size"))?;

        Ok(())
    }
}
