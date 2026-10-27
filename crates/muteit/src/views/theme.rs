// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use iced::border::Radius;
use iced::theme::Style;
use iced::{Background, Border, Color, Length, Padding, Pixels, color};

pub mod color {
    use super::*;

    // oklch(44.6% 0.03 256.802)
    pub const GRAY_600: Color = color!(0x4b5563);
    // oklch(37.3% 0.034 259.733)
    pub const GRAY_700: Color = color!(0x374151);
    // oklch(27.8% 0.033 256.848)
    pub const GRAY_800: Color = color!(0x1f2937);
    // oklch(21% 0.034 264.665)
    pub const GRAY_900: Color = color!(0x111827);

    // neutral-800: oklch(26.9% 0 none)
    pub const NEUTRAL_800: Color = color!(0x262626);

    pub const DANGER: Color = color!(0xe7000b);

    pub const TEXT: Color = Color::WHITE;
}

pub const HEIGHT_PIXELS: Pixels = Pixels(44.);
pub const HEIGHT: Length = Length::Fixed(HEIGHT_PIXELS.0);

pub const TEXT_SM: Pixels = Pixels(14.);

pub const PADDING: Padding = Padding {
    left: 14.,
    right: 14.,
    top: 0.,
    bottom: 0.,
};

pub const BACKGROUND: Background = Background::Color(Color::BLACK);

pub const BORDER_RADIUS: Radius = Radius {
    bottom_left: 6.,
    bottom_right: 6.,
    top_left: 6.,
    top_right: 6.,
};
pub const BORDER: Border = Border {
    color: color::GRAY_900,
    radius: BORDER_RADIUS,
    width: 2.,
};
pub const HOVER_BORDER: Border = Border {
    color: color::GRAY_700,
    radius: BORDER_RADIUS,
    width: 2.,
};
pub const PRESSED_BORDER: Border = Border {
    color: color::GRAY_700,
    radius: BORDER_RADIUS,
    width: 3.,
};

#[inline(always)]
pub fn style() -> Style {
    Style {
        background_color: color::NEUTRAL_800,
        text_color: color::TEXT,
    }
}
