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

use std::env;

use cocoa::appkit::CGFloat;
use snafu::{ResultExt, whatever};
use unnamed::{
    BundleID, UnnamedError, get_screen_frame, has_accessibility_permissions,
    running_apps_with_bundle_id,
};

const LEFT_INSET: CGFloat = 8.0;
const RIGHT_INSET: CGFloat = 8.0;
const TOP_INSET: CGFloat = 6.0;
const BOTTOM_INSET: CGFloat = 8.0;

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

    let mut frame = get_screen_frame();

    frame.origin.x += LEFT_INSET;
    frame.origin.y += TOP_INSET;
    frame.size.width -= LEFT_INSET + RIGHT_INSET;
    frame.size.height -= TOP_INSET + BOTTOM_INSET;

    for bundle_id in bundle_ids {
        for app in running_apps_with_bundle_id(bundle_id)? {
            for mut window in app.get_windows()? {
                window.resize(frame)?;
            }
        }
    }

    Ok(())
}
