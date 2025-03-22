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
    kAXTrustedCheckOptionPrompt, AXIsProcessTrustedWithOptions,
};
use core_foundation_sys::{
    base::{kCFAllocatorDefault, CFRelease},
    dictionary::CFDictionaryCreate,
    number::kCFBooleanTrue,
};
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum WiseError {
    #[snafu(display("Failed to create or copy CoreFoundation object"))]
    CouldNotCreateCFObject,
}

pub fn has_accessibility_permissions() -> Result<bool, WiseError> {
    // SAFETY:
    // `kAXTrustedCheckOptionPrompt` should be initialized by CoreFoundation.
    let keys = [unsafe { kAXTrustedCheckOptionPrompt } as *const _];

    // SAFETY:
    // `kCFBooleanTrue` should be initialized by CoreFoundation.
    let values = [unsafe { kCFBooleanTrue } as *const _];

    // SAFETY:
    // - `keys.as_ptr()` is a valid pointer to a C array of at least 1
    //   pointer-sized value.
    // - `values.as_ptr()` is likewise.
    let options = unsafe {
        CFDictionaryCreate(
            kCFAllocatorDefault,
            keys.as_ptr(),
            values.as_ptr(),
            1,
            ptr::null(),
            ptr::null(),
        )
    };

    if options.is_null() {
        return Err(WiseError::CouldNotCreateCFObject);
    }

    // SAFETY:
    // `options` is a valid dictionary of options.
    let is_trusted = unsafe { AXIsProcessTrustedWithOptions(options) };

    // SAFETY:
    // `options` is non-null and was just allocated with a Create function.
    unsafe { CFRelease(options as *const _) };

    Ok(is_trusted)
}

pub fn running_apps_with_bundle_id() {}
