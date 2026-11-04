// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::borrow::Borrow;

use iced::font::Weight;
use iced::widget::text::LineHeight;
use iced::widget::{Button, Column, Container, PickList, Space, Text, Toggler, column, row};
use iced::{Background, Border, Color, Element, Font, Length, Padding};

use crate::views::theme::{
    BACKGROUND, BORDER, HEIGHT, HEIGHT_PIXELS, HOVER_BORDER, PADDING, PRESSED_BORDER, SHADOW,
    TEXT_SM, color, font,
};
use crate::views::widgets::absolute::Absolute;

pub fn label<'a, Message, Theme, Renderer>(
    content: &'a str,
) -> Container<'a, Message, Theme, Renderer>
where
    Theme: iced::widget::container::Catalog + iced::widget::text::Catalog + 'a,
    Renderer: iced::advanced::text::Renderer + 'a,
    <Renderer as iced::advanced::text::Renderer>::Font: From<iced::Font>,
{
    Container::new(Text::new(content).size(TEXT_SM).font(Font {
        weight: Weight::Bold,
        ..Font::DEFAULT
    }))
    .padding(Padding::default().left(4).bottom(2))
}

pub fn button<'a, Message, Theme, Renderer>(
    content: &'a str,
) -> Button<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: iced::widget::button::Catalog + iced::widget::text::Catalog + 'a,
    Renderer: iced::advanced::text::Renderer + 'a,
    <Theme as iced::widget::button::Catalog>::Class<'a>:
        From<iced::widget::button::StyleFn<'a, Theme>>,
{
    use iced::widget::button::{Status, Style};

    Button::new(Text::new(content).height(Length::Fill).center())
        .style(|_, status| {
            let border = match status {
                Status::Active | Status::Disabled => BORDER,
                Status::Hovered => HOVER_BORDER,
                Status::Pressed => PRESSED_BORDER,
            };

            Style {
                background: Some(BACKGROUND),
                text_color: color::TEXT,
                shadow: SHADOW,
                border,
                ..Default::default()
            }
        })
        .height(HEIGHT)
        .padding(PADDING)
}

pub fn toggler<'a, Message, Theme, Renderer>(
    is_toggled: bool,
) -> Toggler<'a, Message, Theme, Renderer>
where
    Theme: iced::widget::toggler::Catalog,
    Renderer: iced::advanced::text::Renderer,
    <Theme as iced::widget::toggler::Catalog>::Class<'a>:
        From<iced::widget::toggler::StyleFn<'a, Theme>>,
{
    use iced::widget::toggler::{Status, Style};

    #[inline]
    const fn style(is_toggled: bool, is_hovered: bool) -> Style {
        let foreground_color = if is_toggled {
            color::TEXT
        } else {
            color::NEUTRAL_700
        };

        let background_border_color = if is_hovered {
            HOVER_BORDER.color
        } else {
            BORDER.color
        };

        Style {
            foreground: Background::Color(foreground_color),
            foreground_border_color: foreground_color,
            foreground_border_width: 0.,
            background: BACKGROUND,
            background_border_color,
            background_border_width: BORDER.width,
            padding_ratio: 0.15,
            border_radius: None,
            text_color: None,
        }
    }

    Toggler::new(is_toggled)
        .style(|_, status| match status {
            Status::Active { is_toggled } | Status::Disabled { is_toggled } => {
                style(is_toggled, false)
            }
            Status::Hovered { is_toggled } => style(is_toggled, true),
        })
        .size(HEIGHT_PIXELS.0 - 4.)
}

pub fn option<'a, Message, Theme, Renderer>(
    label: &'a str,
    is_toggled: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Column<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: iced::widget::text::Catalog
        + iced::widget::container::Catalog
        + iced::widget::toggler::Catalog
        + 'a,
    Renderer: iced::advanced::text::Renderer + 'a,
    <Renderer as iced::advanced::text::Renderer>::Font: From<iced::Font>,
    <Theme as iced::widget::text::Catalog>::Class<'a>: From<iced::widget::text::StyleFn<'a, Theme>>,
    <Theme as iced::widget::toggler::Catalog>::Class<'a>:
        From<iced::widget::toggler::StyleFn<'a, Theme>>,
{
    column![
        Space::new().height(10.),
        row![
            Text::new(label)
                .font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                })
                .color(color::TEXT)
                .height(Length::Fill)
                .width(Length::Fill)
                .align_y(iced::Alignment::Center),
            Container::new(toggler(is_toggled).on_toggle(on_toggle)).center_y(Length::Fill)
        ]
        .height(HEIGHT)
    ]
}

pub fn pick_list<'a, T, L, V, Message, Theme, Renderer>(
    options: L,
    selected: Option<V>,
    on_selected: impl Fn(T) -> Message + 'a,
) -> Container<'a, Message, Theme, Renderer>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Message: Clone + 'a,
    Theme: iced::widget::pick_list::Catalog
        + iced::widget::overlay::menu::Catalog
        + iced::widget::container::Catalog
        + 'a,
    Renderer: iced::advanced::text::Renderer + 'a,
    <Theme as iced::widget::pick_list::Catalog>::Class<'a>:
        From<iced::widget::pick_list::StyleFn<'a, Theme>>,
    <Theme as iced::overlay::menu::Catalog>::Class<'a>:
        From<iced::widget::overlay::menu::StyleFn<'a, Theme>>,
{
    use iced::widget::overlay::menu;
    use iced::widget::pick_list::{Status, Style};

    Container::new(
        PickList::new(options, selected, on_selected)
            .style(|_, status| {
                let border = match status {
                    Status::Active | Status::Opened { is_hovered: false } => BORDER,
                    Status::Hovered | Status::Opened { is_hovered: true } => HOVER_BORDER,
                };

                Style {
                    background: BACKGROUND,
                    handle_color: color::TEXT,
                    placeholder_color: color::TEXT,
                    text_color: color::TEXT,
                    shadow: SHADOW,
                    border,
                }
            })
            .menu_style(|_| menu::Style {
                background: Background::Color(color::NEUTRAL_850),
                selected_background: Background::Color(color::NEUTRAL_750),
                selected_text_color: color::TEXT,
                text_color: color::TEXT,
                border: BORDER.color(color::NEUTRAL_900),
                shadow: Default::default(),
            })
            .width(Length::Fill)
            .text_line_height(LineHeight::Absolute(HEIGHT_PIXELS))
            .padding(PADDING),
    )
    .height(HEIGHT)
}

pub fn kbd<'a, Message>(content: &'a str) -> Element<'a, Message>
where
    Message: 'a,
{
    use iced::widget::container::Style;

    Container::new(Text::new(content).size(TEXT_SM).font(font::MONO))
        .style(|_| Style {
            background: Some(Background::Color(Color::BLACK)),
            border: Border {
                color: color::GRAY_600,
                width: 1.2,
                ..BORDER
            },
            text_color: Some(color::TEXT),
            ..Default::default()
        })
        .center_y(Length::Fill)
        .padding(Padding::default().horizontal(10))
        .into()
}

pub fn absolute<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    absolute_content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Absolute<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    Absolute::new(content, absolute_content)
}
