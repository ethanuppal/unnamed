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

use accessibility_sys::{
    AXValueCreate, AXValueRef, kAXValueTypeCGPoint, kAXValueTypeCGSize,
};
use cocoa::{
    appkit::{CGFloat, CGPoint, NSScreen},
    base::nil,
};
use core_graphics::display::{CGRect, CGSize};

use crate::{UnnamedError, memory::Unique};

pub struct AXRect {
    pub origin: Unique<AXValueRef>,
    pub size: Unique<AXValueRef>,
}

const LEFT_INSET: CGFloat = 8.0;
const RIGHT_INSET: CGFloat = 8.0;
const TOP_INSET: CGFloat = 6.0;
const BOTTOM_INSET: CGFloat = 8.0;
const INNER_SPACING: CGFloat = 12.0;

#[derive(Default, Clone, Copy)]
#[repr(usize)]
pub enum LayoutPreset {
    #[default]
    Full,
    Left,
    Right,
    COUNT,
}

pub struct LayoutPresets {
    pub rects: [AXRect; LayoutPreset::COUNT as usize],
}

fn create_ax_rect(frame: CGRect) -> Result<AXRect, UnnamedError> {
    // SAFETY: ``&full_frame.origin` is a valid pointer and not mutably
    // referenced throughout the course of this function.
    let ax_origin = unsafe {
        Unique::new_mut(AXValueCreate(
            kAXValueTypeCGPoint,
            &frame.origin as *const CGPoint as *const _,
        ))
    }
    .ok_or(UnnamedError::CouldNotCreateCFObject)?;

    // SAFETY: ``&full_frame.size` is a valid pointer and not mutably
    // referenced throughout the course of this function.
    let ax_size = unsafe {
        Unique::new_mut(AXValueCreate(
            kAXValueTypeCGSize,
            &frame.size as *const CGSize as *const _,
        ))
    }
    .ok_or(UnnamedError::CouldNotCreateCFObject)?;

    Ok(AXRect {
        origin: ax_origin,
        size: ax_size,
    })
}

fn split_horizontal(frame: CGRect) -> (CGRect, CGRect) {
    let half_width = frame.size.width / 2.0;

    let mut left = frame;
    left.size.width = half_width;

    let mut right = frame;
    right.origin.x += half_width;
    right.size.width = half_width;

    (left, right)
}

fn inset(
    mut rect: CGRect,
    left: CGFloat,
    right: CGFloat,
    top: CGFloat,
    bottom: CGFloat,
) -> CGRect {
    rect.origin.x += left;
    rect.size.width -= left + right;

    rect.origin.y += top;
    rect.size.height -= top + bottom;

    rect
}

pub fn get_layout_presets() -> Result<LayoutPresets, UnnamedError> {
    // SAFETY: todo
    let main_screen = unsafe { NSScreen::mainScreen(nil) };

    const NOTCH_HEIGHT: CGFloat = 40.0;

    let frame = {
        // SAFETY: todo
        let frame_nsrect = unsafe { main_screen.frame() };

        CGRect {
            origin: CGPoint::new(
                frame_nsrect.origin.x,
                frame_nsrect.origin.y + NOTCH_HEIGHT,
            ),
            size: CGSize::new(
                frame_nsrect.size.width,
                frame_nsrect.size.height - NOTCH_HEIGHT,
            ),
        }
    };

    let (left_frame, right_frame) = split_horizontal(frame);

    Ok(LayoutPresets {
        rects: [
            create_ax_rect(inset(
                frame,
                LEFT_INSET,
                RIGHT_INSET,
                TOP_INSET,
                BOTTOM_INSET,
            ))?,
            create_ax_rect(inset(
                left_frame,
                LEFT_INSET,
                INNER_SPACING / 2.0,
                TOP_INSET,
                BOTTOM_INSET,
            ))?,
            create_ax_rect(inset(
                right_frame,
                INNER_SPACING / 2.0,
                RIGHT_INSET,
                TOP_INSET,
                BOTTOM_INSET,
            ))?,
        ],
    })
}
