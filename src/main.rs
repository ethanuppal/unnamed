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

use std::{
    collections::HashSet,
    env, ffi,
    ptr::{self},
    sync::LazyLock,
};

use accessibility_sys::{
    AXObserverAddNotification, AXObserverCreate, AXObserverGetRunLoopSource,
    AXObserverRef, AXUIElementRef, kAXWindowMovedNotification,
    kAXWindowResizedNotification,
};
use cocoa::{appkit::NSWorkspace, base::nil};
use core_foundation_sys::{
    runloop::{CFRunLoopAddSource, CFRunLoopGetCurrent, kCFRunLoopDefaultMode},
    string::CFStringRef,
};
use dashmap::DashMap;
use rdev::{EventType, Key};
use snafu::{ResultExt, whatever};
use unnamed::{
    AXErrorExt, BundleID, UnnamedError, has_accessibility_permissions,
    layout::{Layout, Layouts, get_layouts},
    memory::{CopyOnWrite, Unique},
    running_apps_with_bundle_id,
    wrappers::{
        AccessibilityElement, App, Window, create_cfstring_from_static_str,
    },
};

static LAYOUT_ASSIGNMENTS: LazyLock<DashMap<String, (Layout, bool)>> =
    LazyLock::new(DashMap::new);

unsafe extern "C" fn observer_callback(
    _observer: AXObserverRef,
    element: AXUIElementRef,
    _notification: CFStringRef,
    refcon: *mut ffi::c_void,
) {
    // SAFETY: todo
    let layouts = unsafe { (refcon as *const _ as *const Layouts).as_ref() }
        .expect("Got passed null?");
    //println!("resize: {element:?} {_notification:?}");

    //println!("tryign to print");
    // SAFETY: todo
    //println!("{}", unsafe { CFGetRetainCount(element as CFTypeRef) });

    // SAFETY: todo
    let mut window = unsafe { Window::borrow_inner(element) }
        .expect("Window observer should be passed valid window");

    let (layout, enabled) =
        *LAYOUT_ASSIGNMENTS.get(window.bundle_id().as_ref()).unwrap();
    if enabled {
        window
            .relayout(&layouts.rects[layout as usize])
            .expect("Failed to relayout window");
    }
}

#[derive(Default)]
struct KeyState {
    keys_down: HashSet<Key>,
}

impl KeyState {
    fn press(&mut self, key: Key) {
        self.keys_down.insert(key);
    }

    fn release(&mut self, key: &Key) {
        self.keys_down.remove(key);
    }

    fn is_modifier_down(&self) -> bool {
        let command = self.keys_down.contains(&Key::MetaLeft)
            || self.keys_down.contains(&Key::MetaRight);
        let control = self.keys_down.contains(&Key::ControlLeft)
            || self.keys_down.contains(&Key::ControlRight);
        let option = self.keys_down.contains(&Key::Alt)
            || self.keys_down.contains(&Key::AltGr);
        let shift = self.keys_down.contains(&Key::ShiftLeft)
            || self.keys_down.contains(&Key::ShiftRight);
        command && control && option && shift
    }

    fn is_modified(&self, key: Key) -> bool {
        self.is_modifier_down() && self.keys_down.contains(&key)
    }
}

fn update_layout_for_focused_window(
    new_layout: Option<Layout>,
    layouts: &Layouts,
) -> Result<(), UnnamedError> {
    // SAFETY: todo
    let workspace = unsafe { NSWorkspace::sharedWorkspace(nil) };
    if workspace.is_null() {
        return Err(UnnamedError::UnexpectedNull);
    }
    //println!("test");

    // SAFETY: todo
    let app = unsafe { NSWorkspace::frontmostApplication(workspace) };
    if app.is_null() {
        return Err(UnnamedError::UnexpectedNull);
    }

    // SAFETY: todo
    let app = unsafe { App::from_nsapp(CopyOnWrite::Borrowed(app), None) }?;

    //println!("{}", app.bundle_id().as_ref());
    if let Some(new_layout) = new_layout {
        *LAYOUT_ASSIGNMENTS
            .get_mut(app.bundle_id().as_ref())
            .unwrap() = (new_layout, true);
    } else {
        LAYOUT_ASSIGNMENTS
            .get_mut(app.bundle_id().as_ref())
            .unwrap()
            .1 ^= true;
    }

    for mut window in app.get_windows()? {
        // TODO: code duplication
        let (layout, enabled) =
            *LAYOUT_ASSIGNMENTS.get(window.bundle_id().as_ref()).unwrap();
        if enabled {
            if let Err(error) = window.relayout(&layouts.rects[layout as usize])
            {
                eprintln!("error: {error}");
            }
        }
    }

    Ok(())
}

#[snafu::report]
fn main() -> Result<(), UnnamedError> {
    let args = env::args().collect::<Vec<_>>();
    let args = args
        .iter()
        .map(|string| string.as_str())
        .collect::<Vec<_>>();

    let bundle_ids = match (args.as_slice(), args.len()) {
        (&[_, "--help"], 2) => {
            println!(
                "usage: {} <bundle IDs> | {0} --help | {0} --version",
                args[0]
            );
            return Ok(());
        }
        (&[_, "--version"], 2) => {
            println!("{} {}", args[0], env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        (other, args) if args > 1 => other
            .iter()
            .skip(1)
            .cloned()
            .map(BundleID::try_from)
            .collect::<Result<Vec<_>, _>>()
            .whatever_context("Failed to parse provided bundle IDs")?,
        _ => {
            whatever!("Invalid arguments. Pass --help for usage information.");
        }
    };

    if !has_accessibility_permissions()? {
        whatever!("This program needs accessibility permissions to work");
    }

    let layouts =
        get_layouts().whatever_context("Failed to compute layouts")?;

    let mut observers = vec![];

    for bundle_id in bundle_ids {
        LAYOUT_ASSIGNMENTS.insert(bundle_id.to_string(), (Layout::Full, true));

        for app in running_apps_with_bundle_id(bundle_id)? {
            let mut observer = ptr::null_mut();
            // SAFETY: todo
            unsafe {
                AXObserverCreate(app.pid(), observer_callback, &mut observer)
            }
            .into_result()?;
            // SAFETY: todo
            let observer = unsafe { Unique::new_mut(observer) }
                .ok_or(UnnamedError::UnexpectedNull)?;

            for mut window in app.get_windows()? {
                window.relayout(&layouts.rects[Layout::Full as usize])?;

                let notification = create_cfstring_from_static_str(
                    kAXWindowResizedNotification,
                )?;

                // SAFETY: todo
                unsafe {
                    AXObserverAddNotification(
                        observer.get(),
                        window.inner(),
                        notification.get(),
                        &layouts as *const _ as *mut _,
                    )
                }
                .into_result()
                .whatever_context(format!(
                    "Failed to observe window resizes in {bundle_id}"
                ))?;

                let notification = create_cfstring_from_static_str(
                    kAXWindowMovedNotification,
                )?;

                // SAFETY: todo
                unsafe {
                    AXObserverAddNotification(
                        observer.get(),
                        window.inner(),
                        notification.get(),
                        &layouts as *const _ as *mut _,
                    )
                }
                .into_result()
                .whatever_context(format!(
                    "Failed to observe window moves in {bundle_id}"
                ))?;
            }

            // SAFETY: todo
            let run_loop_source =
                unsafe { AXObserverGetRunLoopSource(observer.get()) };
            if run_loop_source.is_null() {
                return Err(UnnamedError::UnexpectedNull);
            }
            // SAFETY: todo
            unsafe {
                CFRunLoopAddSource(
                    CFRunLoopGetCurrent(),
                    run_loop_source,
                    kCFRunLoopDefaultMode,
                )
            };

            observers.push(observer);
        }
    }

    let mut key_state = KeyState::default();

    // rdev automatically sets up the CGRunLoop
    rdev::listen(move |event| match event.event_type {
        EventType::KeyPress(key) => {
            key_state.press(key);

            if let Some(new_layout) = if key_state.is_modified(Key::KeyH) {
                Some(Some(Layout::Left))
            } else if key_state.is_modified(Key::KeyL) {
                Some(Some(Layout::Right))
            } else if key_state.is_modified(Key::KeyC) {
                Some(Some(Layout::Full))
            } else if key_state.is_modified(Key::Space) {
                Some(None)
            } else {
                //if key_state.is_modified(Key::Space) { todo figure out toggle
                None
            } {
                update_layout_for_focused_window(new_layout, &layouts)
                    .expect("Failed to update window layouts");
            }
        }
        EventType::KeyRelease(key) => {
            key_state.release(&key);
        }
        _ => {}
    })
    .map_err(|inner| UnnamedError::RDevError { inner })
    .whatever_context("CGRunLoop failed")?;

    Ok(())
}
