// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Shell, Widget, layout, mouse, renderer};
use iced::{Element, Length, Point, Rectangle, Size, Vector};

/// A position relative to something else.
///
/// # Examples
///
/// For a `widget` in a `view` and a position
/// `RelativePosition::TopRight(Vector { x: 1., y: -2.})`.
///
/// If the position is relative to `widget`, you'll get:
///
/// ```text
/// +view----------------------+
/// |                      TopRight(Vector { x: 1., y: -2. })
/// |                     x    |
/// |                          |
/// |     +widget--------+     |
/// |     |              |     |
/// |     |              |     |
/// |     |              |     |
/// |     |              |     |
/// |     +--------------+     |
/// +--------------------------+
/// ```
#[derive(Debug, Clone, Copy)]
pub enum RelativePosition {
    /// Relative position from the top-left of the content.
    TopLeft(Vector),
    /// Relative position from the top-right of the content.
    TopRight(Vector),
    /// Relative position from the bottom-left of the content.
    BottomLeft(Vector),
    /// Relative position from the bottom-right of the content.
    BottomRight(Vector),
}

impl Default for RelativePosition {
    fn default() -> Self {
        Self::TopLeft(Vector::ZERO)
    }
}

/// Position a widget relative to another, without taking space in the page layout.
///
/// This is analogous to the css [`position: absolute`] property.
///
/// [`position: absolute`]:
///     https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/position#absolute
pub struct Absolute<'a, Message, Theme, Renderer> {
    /// The content.
    content: Element<'a, Message, Theme, Renderer>,
    /// The content with the `absolute` position.
    absolute_content: Element<'a, Message, Theme, Renderer>,
    /// The position of [`absolute_content`](Self::absolute_content), relative to
    /// [`content`](Self::content).
    position: RelativePosition,
}

impl<'a, Message, Theme, Renderer> Absolute<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    /// Create a new [`Absolute`] which will render `content` normally, and render and place
    /// `absolute_content` depending on its relative position to `content`.
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        absolute_content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Absolute {
            content: content.into(),
            absolute_content: absolute_content.into(),
            position: RelativePosition::default(),
        }
    }

    /// Set the relative position of [`absolute_content`](Self::absolute_content) (relative to
    /// [`content`](Self::content)).
    pub fn position(mut self, position: RelativePosition) -> Self {
        self.position = position;
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Absolute<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn children(&self) -> Vec<Tree> {
        vec![
            tree::Tree::new(&self.content),
            tree::Tree::new(&self.absolute_content),
        ]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.content, &self.absolute_content]);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // We only want `content` to be part of the normal flow, so that's the only one we return,
        // but we still need to compute the layout of `absolute_content` so we can properly place
        // it relative to `content` in `update`/`draw`/etc.

        let content_layout_node =
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits);

        let state = tree.state.downcast_mut::<State>();
        state.absolute_layout_node = {
            let position = {
                let bounds = content_layout_node.bounds();
                let content_position = Point::new(bounds.x, bounds.y);

                match self.position {
                    RelativePosition::TopLeft(absolute_position) => {
                        content_position + absolute_position
                    }
                    RelativePosition::TopRight(absolute_position) => Point::new(
                        content_position.x + bounds.width + absolute_position.x,
                        content_position.y + absolute_position.y,
                    ),
                    RelativePosition::BottomLeft(absolute_position) => Point::new(
                        content_position.x + absolute_position.x,
                        content_position.y + bounds.height + absolute_position.y,
                    ),
                    RelativePosition::BottomRight(absolute_position) => Point::new(
                        content_position.x + bounds.width + absolute_position.x,
                        content_position.y + bounds.height + absolute_position.y,
                    ),
                }
            };

            self.absolute_content
                .as_widget_mut()
                .layout(&mut tree.children[1], renderer, &layout::Limits::NONE)
                .move_to(position)
        };

        content_layout_node
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: layout::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );

            let state = tree.state.downcast_ref::<State>();

            self.absolute_content.as_widget_mut().operate(
                &mut tree.children[1],
                state.absolute_layout(&layout),
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let state = tree.state.downcast_ref::<State>();

        self.absolute_content.as_widget_mut().update(
            &mut tree.children[1],
            event,
            state.absolute_layout(&layout),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();

        let interaction = self.absolute_content.as_widget().mouse_interaction(
            &tree.children[1],
            state.absolute_layout(&layout),
            cursor,
            viewport,
            renderer,
        );

        // `absolute_content` will be rendered on top of `content`, so check for its interaction
        // first.
        if interaction != mouse::Interaction::None {
            interaction
        } else {
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );

        let state = tree.state.downcast_ref::<State>();

        renderer.with_layer(*viewport, |renderer| {
            self.absolute_content.as_widget().draw(
                &tree.children[1],
                renderer,
                theme,
                style,
                state.absolute_layout(&layout),
                cursor,
                viewport,
            )
        });
    }
}

impl<'a, Message, Theme, Renderer> From<Absolute<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(widget: Absolute<'a, Message, Theme, Renderer>) -> Self {
        Self::new(widget)
    }
}

#[derive(Default)]
struct State {
    /// The layout [`Node`](layout::Node) of [`Absolute::absolute_content`].
    //
    // NOTE: `Layout` borrows its node. If we keep the node in `Absolute`, we won't be able to use
    // it in `&mut self` `Widget` methods because of the borrow checker. Cloning it could be costly
    // (it contains a vec), so we keep it here instead.
    absolute_layout_node: layout::Node,
}

impl State {
    fn new() -> Self {
        Self::default()
    }

    /// Get the [`Layout`](layout::Layout) of [`Absolute::absolute_content`].
    fn absolute_layout(&self, layout: &layout::Layout<'_>) -> layout::Layout<'_> {
        let offset = {
            let position = layout.position();

            Vector::new(position.x, position.y)
        };

        layout::Layout::with_offset(offset, &self.absolute_layout_node)
    }
}
