//! A Kindle-friendly book card.
//!
//! This is intentionally a component primitive rather than an app-specific
//! drawing shortcut: Ember owns its measurement, retained paint state, and
//! hit-testing just as it does for [`Button`](super::Button).

use std::cell::RefCell;
use std::rc::Rc;

use crate::geometry::{Rect, Size};
use crate::render::{Color, DrawCmd};
use crate::widget::{font, Common};

const PAD: i32 = 10;
const CARD_W: i32 = 220;
const CARD_H: i32 = 372;
const COVER_H: i32 = 278;
const LIST_W: i32 = 720;
const LIST_H: i32 = 188;
const LIST_COVER_W: i32 = 108;
const LIST_COVER_H: i32 = 156;

pub(crate) struct BookCardState {
    pub common: Common,
    pub title: String,
    pub subtitle: Option<String>,
    pub detail: Option<String>,
    pub progress: Option<u8>,
    pub cover: Option<(i32, i32, Rc<Vec<u8>>)>,
    pub featured: bool,
    /// The mobile product surface uses a horizontal, cover-leading card.  It
    /// deliberately lives in the reusable primitive rather than in an app
    /// renderer so it retains Ember layout and hit-testing semantics.
    pub horizontal: bool,
    pub on_tap: Option<Rc<dyn Fn()>>,
}

impl BookCardState {
    pub(crate) fn measure(&self, avail: Size) -> Size {
        if self.horizontal {
            return Size::new(
                LIST_W.min(avail.w.max(0)),
                if self.featured { LIST_H + 24 } else { LIST_H },
            );
        }
        let h = if self.featured { CARD_H + 34 } else { CARD_H };
        Size::new(CARD_W.min(avail.w.max(0)), h)
    }

    pub(crate) fn paint(&self, out: &mut Vec<DrawCmd>) {
        let f = self.common.frame;
        if self.horizontal {
            return self.paint_horizontal(out);
        }
        let cover_h = if self.featured {
            (COVER_H + 28).min(f.h - 88)
        } else {
            COVER_H.min(f.h - 82)
        };
        let cover_x = f.x + PAD;
        let cover_y = f.y + PAD;
        let cover_w = (f.w - PAD * 2).max(1);
        let title_y = cover_y + cover_h + 10;
        let title_size = if self.featured { 3 } else { 2 };

        out.push(DrawCmd::FillRect {
            rect: f,
            color: Color(15),
        });
        let cover_rect = Rect::new(cover_x, cover_y, cover_w, cover_h);
        if let Some((width, height, pixels)) = &self.cover {
            out.push(DrawCmd::Image {
                rect: cover_rect,
                width: *width,
                height: *height,
                pixels: pixels.clone(),
            });
        } else {
            out.push(DrawCmd::FillRect {
                rect: cover_rect,
                color: Color(12),
            });
            let placeholder = "NO COVER";
            let placeholder_size = 2;
            let measured = font::text_size(placeholder, placeholder_size);
            out.push(DrawCmd::Text {
                x: cover_x + (cover_w - measured.w) / 2,
                y: cover_y + (cover_h - measured.h) / 2,
                text: placeholder.into(),
                size: placeholder_size,
                fg: Color::BLACK,
                bg: Color(12),
                inverse: false,
            });
        }
        out.push(DrawCmd::Text {
            x: cover_x,
            y: title_y,
            text: self.title.clone(),
            size: title_size,
            fg: Color::BLACK,
            bg: Color(15),
            inverse: false,
        });
        if let Some(subtitle) = &self.subtitle {
            out.push(DrawCmd::Text {
                x: cover_x,
                y: title_y + title_size as i32 * font::CELL_H + 4,
                text: subtitle.clone(),
                size: 2,
                // E-ink panels turn the stock muted palette nearly white.
                // Secondary book metadata must remain readable in daylight.
                fg: Color::BLACK,
                bg: Color(15),
                inverse: false,
            });
        }
        if let Some(detail) = &self.detail {
            out.push(DrawCmd::Text {
                x: cover_x,
                y: f.bottom() - PAD - font::CELL_H * 2,
                text: detail.clone(),
                size: 2,
                fg: Color::BLACK,
                bg: Color(15),
                inverse: false,
            });
        }
        if let Some(progress) = self.progress {
            let width = (cover_w * progress.min(100) as i32) / 100;
            out.push(DrawCmd::FillRect {
                rect: Rect::new(cover_x, cover_y + cover_h - 5, cover_w, 5),
                color: Color(10),
            });
            out.push(DrawCmd::FillRect {
                rect: Rect::new(cover_x, cover_y + cover_h - 5, width, 5),
                color: Color(3),
            });
        }
    }

    fn paint_horizontal(&self, out: &mut Vec<DrawCmd>) {
        let f = self.common.frame;
        let cover_x = f.x + PAD;
        let cover_y = f.y + PAD;
        let cover_h = (f.h - PAD * 2).clamp(1, LIST_COVER_H);
        let cover_w = (cover_h * LIST_COVER_W / LIST_COVER_H).max(1);
        let text_x = cover_x + cover_w + 18;
        let title_size = 3;

        out.push(DrawCmd::FillRect {
            rect: f,
            color: Color::WHITE,
        });
        stroke_rect(out, f, 2, Color(12));
        let cover_rect = Rect::new(cover_x, cover_y, cover_w, cover_h);
        if let Some((width, height, pixels)) = &self.cover {
            out.push(DrawCmd::Image {
                rect: cover_rect,
                width: *width,
                height: *height,
                pixels: pixels.clone(),
            });
        } else {
            out.push(DrawCmd::FillRect {
                rect: cover_rect,
                color: Color(12),
            });
            let placeholder = "NO COVER";
            let placeholder_size = 1;
            let measured = font::text_size(placeholder, placeholder_size);
            out.push(DrawCmd::Text {
                x: cover_x + (cover_w - measured.w) / 2,
                y: cover_y + (cover_h - measured.h) / 2,
                text: placeholder.into(),
                size: placeholder_size,
                fg: Color::BLACK,
                bg: Color(12),
                inverse: false,
            });
        }
        stroke_rect(out, cover_rect, 2, Color(8));
        out.push(DrawCmd::Text {
            x: text_x,
            y: cover_y + 8,
            text: self.title.clone(),
            size: title_size,
            fg: Color::BLACK,
            bg: Color(15),
            inverse: false,
        });
        if let Some(subtitle) = &self.subtitle {
            out.push(DrawCmd::Text {
                x: text_x,
                y: cover_y + 16 + title_size as i32 * font::CELL_H,
                text: subtitle.clone(),
                size: 2,
                fg: Color::BLACK,
                bg: Color(15),
                inverse: false,
            });
        }
        if let Some(detail) = &self.detail {
            out.push(DrawCmd::Text {
                x: text_x,
                y: f.bottom() - PAD - font::CELL_H * 2 - 12,
                text: detail.clone(),
                size: 2,
                fg: Color::BLACK,
                bg: Color(15),
                inverse: false,
            });
        }
        if let Some(progress) = self.progress {
            let rail_w = (f.right() - text_x - PAD).clamp(1, 620);
            let width = rail_w * progress.min(100) as i32 / 100;
            let rail_y = f.bottom() - PAD - 4;
            out.push(DrawCmd::FillRect {
                rect: Rect::new(text_x, rail_y, rail_w, 4),
                color: Color(12),
            });
            out.push(DrawCmd::FillRect {
                rect: Rect::new(text_x, rail_y, width, 4),
                color: Color::BLACK,
            });
        }
    }
}

fn stroke_rect(out: &mut Vec<DrawCmd>, rect: Rect, width: i32, color: Color) {
    let width = width.max(1).min(rect.w.max(1)).min(rect.h.max(1));
    out.push(DrawCmd::FillRect {
        rect: Rect::new(rect.x, rect.y, rect.w, width),
        color,
    });
    out.push(DrawCmd::FillRect {
        rect: Rect::new(rect.x, rect.bottom() - width, rect.w, width),
        color,
    });
    out.push(DrawCmd::FillRect {
        rect: Rect::new(rect.x, rect.y, width, rect.h),
        color,
    });
    out.push(DrawCmd::FillRect {
        rect: Rect::new(rect.right() - width, rect.y, width, rect.h),
        color,
    });
}

/// A tappable book-shaped card with a cover placeholder and reading progress.
#[derive(Clone)]
pub struct BookCard(pub(crate) Rc<RefCell<BookCardState>>);

impl BookCard {
    pub fn new(title: impl Into<String>) -> Self {
        Self(Rc::new(RefCell::new(BookCardState {
            common: Common::new(),
            title: title.into(),
            subtitle: None,
            detail: None,
            progress: None,
            cover: None,
            featured: false,
            horizontal: false,
            on_tap: None,
        })))
    }

    pub fn subtitle(self, value: impl Into<String>) -> Self {
        self.0.borrow_mut().subtitle = Some(value.into());
        self
    }

    pub fn detail(self, value: impl Into<String>) -> Self {
        self.0.borrow_mut().detail = Some(value.into());
        self
    }

    pub fn progress(self, percent: u8) -> Self {
        self.set_progress(Some(percent));
        self
    }

    /// Supplies already-decoded grayscale cover art. The host owns network and
    /// image decoding; Ember only retains and paints the bitmap.
    pub fn cover(self, width: i32, height: i32, pixels: Vec<u8>) -> Self {
        self.set_cover(width, height, pixels);
        self
    }

    /// Replaces the retained cover bitmap. Returns `false` and clears the
    /// cover when the buffer is not exactly `width * height` bytes.
    pub fn set_cover(&self, width: i32, height: i32, pixels: Vec<u8>) -> bool {
        let expected = width.try_into().ok().and_then(|width: usize| {
            height
                .try_into()
                .ok()
                .and_then(|height: usize| width.checked_mul(height))
        });
        let valid = expected == Some(pixels.len());
        let mut state = self.0.borrow_mut();
        state.cover = valid.then(|| (width, height, Rc::new(pixels)));
        state.common.dirty = true;
        valid
    }

    /// Updates or clears reading progress and marks the card dirty.
    pub fn set_progress(&self, percent: Option<u8>) {
        let mut state = self.0.borrow_mut();
        let progress = percent.map(|value| value.min(100));
        if state.progress != progress {
            state.progress = progress;
            state.common.dirty = true;
        }
    }

    pub fn featured(self) -> Self {
        self.0.borrow_mut().featured = true;
        self
    }

    /// Renders this card in the mobile application's cover-leading list form.
    pub fn horizontal(self) -> Self {
        self.0.borrow_mut().horizontal = true;
        self
    }

    pub fn on_tap(self, handler: impl Fn() + 'static) -> Self {
        self.0.borrow_mut().on_tap = Some(Rc::new(handler));
        self
    }

    pub(crate) fn fire_tap(&self) -> bool {
        let handler = self.0.borrow().on_tap.clone();
        if let Some(handler) = handler {
            handler();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_progress_and_emits_a_valid_image_command() {
        let card = BookCard::new("A Book")
            .progress(200)
            .cover(2, 2, vec![0, 64, 128, 255]);
        card.0.borrow_mut().common.frame = Rect::new(0, 0, CARD_W, CARD_H);

        let mut commands = Vec::new();
        card.0.borrow().paint(&mut commands);

        assert!(commands.iter().any(|command| matches!(
            command,
            DrawCmd::Image {
                width: 2,
                height: 2,
                pixels,
                ..
            } if pixels.as_slice() == [0, 64, 128, 255]
        )));
        assert!(commands.iter().any(|command| matches!(
            command,
            DrawCmd::FillRect { rect, color: Color(3) }
                if rect.w == CARD_W - PAD * 2
        )));
    }

    #[test]
    fn rejects_malformed_cover_buffers() {
        let card = BookCard::new("A Book").cover(2, 2, vec![0, 1, 2]);
        assert!(card.0.borrow().cover.is_none());

        let card = BookCard::new("A Book").cover(-1, 2, vec![]);
        assert!(card.0.borrow().cover.is_none());
    }

    #[test]
    fn horizontal_cards_use_the_list_measurement() {
        let card = BookCard::new("A Book").horizontal().featured();
        assert_eq!(
            card.0.borrow().measure(Size::new(600, 900)),
            Size::new(600, LIST_H + 24)
        );
    }
}
