// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::borrow::Borrow;

use iced::font::Weight;
use iced::widget::text::LineHeight;
use iced::widget::{Button, Container, PickList, Text};
use iced::{Background, Border, Color, Element, Font, Length, Padding};

use crate::views::theme::{
    BACKGROUND, BORDER, HEIGHT, HEIGHT_PIXELS, HOVER_BORDER, PADDING, PRESSED_BORDER, TEXT_SM,
    color, font,
};

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
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer + 'a,
    <Theme as iced::widget::button::Catalog>::Class<'a>:
        From<iced::widget::button::StyleFn<'a, Theme>>,
{
    use iced::widget::button::{Status, Style};

    Button::new(Text::new(content).height(Length::Fill).center())
        .style(|_, status| match status {
            Status::Active | Status::Disabled => Style {
                background: Some(BACKGROUND),
                text_color: color::TEXT,
                border: BORDER,
                ..Default::default()
            },
            Status::Hovered => Style {
                background: Some(BACKGROUND),
                text_color: color::TEXT,
                border: HOVER_BORDER,
                ..Default::default()
            },
            Status::Pressed => Style {
                background: Some(BACKGROUND),
                text_color: color::TEXT,
                border: PRESSED_BORDER,
                ..Default::default()
            },
        })
        .height(HEIGHT)
        .padding(PADDING)
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
            .style(|_, status| match status {
                Status::Active | Status::Opened { is_hovered: false } => Style {
                    background: BACKGROUND,
                    handle_color: color::TEXT,
                    placeholder_color: color::TEXT,
                    text_color: color::TEXT,
                    border: BORDER,
                },
                Status::Hovered | Status::Opened { is_hovered: true } => Style {
                    background: BACKGROUND,
                    handle_color: color::TEXT,
                    placeholder_color: color::TEXT,
                    text_color: color::TEXT,
                    border: HOVER_BORDER,
                },
            })
            .menu_style(|_| menu::Style {
                background: Background::Color(color::GRAY_800),
                selected_background: Background::Color(color::GRAY_700),
                selected_text_color: color::TEXT,
                text_color: color::TEXT,
                border: BORDER.color(Color::BLACK),
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
            background: Some(BACKGROUND),
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
