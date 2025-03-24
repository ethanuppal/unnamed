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

use cocoa::appkit::CGFloat;
use snafu::whatever;
use wise::{
    WiseError, get_screen_frame, has_accessibility_permissions,
    running_apps_with_bundle_id,
};

const LEFT_INSET: CGFloat = 8.0;
const RIGHT_INSET: CGFloat = 8.0;
const TOP_INSET: CGFloat = 6.0;
const BOTTOM_INSET: CGFloat = 8.0;

#[snafu::report]
fn main() -> Result<(), WiseError> {
    if !has_accessibility_permissions()? {
        whatever!("This program needs accessibility permissions to work");
    }

    let mut frame = get_screen_frame();

    frame.origin.x += LEFT_INSET;
    frame.origin.y += TOP_INSET;
    frame.size.width -= LEFT_INSET + RIGHT_INSET;
    frame.size.height -= TOP_INSET + BOTTOM_INSET;

    for app in running_apps_with_bundle_id("com.apple.Safari")? {
        for mut window in app.get_windows()? {
            window.resize(frame)?;
        }
    }

    Ok(())
}
