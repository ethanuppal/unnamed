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

use cocoa::appkit::CGPoint;
use core_graphics::display::{CGRect, CGSize};
use snafu::whatever;
use wise::{
    WiseError, has_accessibility_permissions, running_apps_with_bundle_id,
};

#[snafu::report]
fn main() -> Result<(), WiseError> {
    if !has_accessibility_permissions()? {
        whatever!("This program needs accessibility permissions to work");
    }

    for app in running_apps_with_bundle_id("com.apple.Safari")? {
        for mut window in app.get_windows()? {
            window.resize(CGRect {
                origin: CGPoint::new(100.0, 100.0),
                size: CGSize::new(400.0, 800.0),
            })?;
        }
    }
    //println!("{} apps", apps.len());
    ////SAFETY: test
    //unsafe {
    //    for app in apps {
    //        println!("retain count: {}", app.strong_count());
    //        println!("pid: {}", app.get().processIdentifier());
    //    }
    //}

    Ok(())
}
