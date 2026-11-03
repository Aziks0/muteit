// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use iced::border::Radius;
use iced::theme::Style;
use iced::{Background, Border, Color, Font, Length, Padding, Pixels, Shadow, Vector, color};

pub mod font {
    use super::*;

    pub const FONT_MONO_BYTES: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/JetBrainsMono-Regular.ttf"
    ));
    pub const MONO: Font = Font::with_name("JetBrains Mono");
}

pub mod color {
    use super::*;

    // oklch(44.6% 0.03 256.802)
    pub const GRAY_600: Color = color!(0x4b5563);

    // Those are based on tailwind but have been tweaked.
    pub const NEUTRAL_700: Color = color!(0x404040);
    pub const NEUTRAL_750: Color = color!(0x333333);
    pub const NEUTRAL_800: Color = color!(0x262626);
    pub const NEUTRAL_850: Color = color!(0x222222);
    pub const NEUTRAL_900: Color = color!(0x1b1b1b);

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

pub const BACKGROUND: Background = Background::Color(color::NEUTRAL_750);

pub const BORDER_RADIUS: Radius = Radius {
    bottom_left: 6.,
    bottom_right: 6.,
    top_left: 6.,
    top_right: 6.,
};
pub const BORDER: Border = Border {
    color: Color::TRANSPARENT,
    radius: BORDER_RADIUS,
    width: 2.,
};
pub const HOVER_BORDER: Border = Border {
    color: color::NEUTRAL_700,
    radius: BORDER_RADIUS,
    width: 2.,
};
pub const PRESSED_BORDER: Border = Border {
    color: color::NEUTRAL_700,
    radius: BORDER_RADIUS,
    width: 3.,
};

pub const SHADOW: Shadow = Shadow {
    color: color::NEUTRAL_850,
    blur_radius: 2.,
    offset: Vector::new(2., 2.),
};

#[inline(always)]
pub fn style() -> Style {
    Style {
        background_color: color::NEUTRAL_800,
        text_color: color::TEXT,
    }
}
