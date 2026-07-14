//! A real bottom tab bar for touch-first e-ink applications.
//!
//! Unlike a row of generic [`Button`](super::Button)s, this widget owns equal
//! tab sizing, icon geometry, centered labels, selection styling, and hit
//! testing as one coherent control.

use std::cell::RefCell;
use std::rc::Rc;

use crate::geometry::{Point, Rect, Size};
use crate::render::{Color, DrawCmd};
use crate::widget::{font, Common};

const HEIGHT: i32 = 108;
const LABEL_SIZE: u8 = 2;

pub(crate) struct TabBarState {
    pub common: Common,
    pub labels: Vec<String>,
    pub active: usize,
    pub on_select: Option<Rc<dyn Fn(usize)>>,
}

impl TabBarState {
    pub(crate) fn measure(&self, avail: Size) -> Size {
        Size::new(avail.w, HEIGHT.min(avail.h.max(0)))
    }

    pub(crate) fn paint(&self, out: &mut Vec<DrawCmd>) {
        let frame = self.common.frame;
        let count = self.labels.len().max(1) as i32;
        let slot_w = frame.w / count;

        out.push(DrawCmd::FillRect {
            rect: frame,
            color: Color::WHITE,
        });
        out.push(DrawCmd::FillRect {
            rect: Rect::new(frame.x, frame.y, frame.w, 2),
            color: Color(10),
        });

        for (index, label) in self.labels.iter().enumerate() {
            let left = frame.x + index as i32 * slot_w;
            let right = if index + 1 == self.labels.len() {
                frame.right()
            } else {
                left + slot_w
            };
            let width = right - left;
            let active = index == self.active;
            if active {
                out.push(DrawCmd::FillRect {
                    rect: Rect::new(left + (width - 84) / 2, frame.y, 84, 5),
                    color: Color::BLACK,
                });
            }

            paint_icon(out, index, left + width / 2, frame.y + 16);
            let label_size = font::text_size(label, LABEL_SIZE);
            out.push(DrawCmd::Text {
                x: left + (width - label_size.w) / 2,
                y: frame.y + 58,
                text: label.clone(),
                size: LABEL_SIZE,
                fg: Color::BLACK,
                bg: Color::WHITE,
                inverse: false,
            });
        }
    }
}

fn paint_icon(out: &mut Vec<DrawCmd>, index: usize, center_x: i32, top: i32) {
    let black = Color::BLACK;
    match index {
        // Reading: two open pages and a center gutter.
        0 => {
            outline_rect(out, Rect::new(center_x - 23, top, 21, 29), 3, black);
            outline_rect(out, Rect::new(center_x + 2, top, 21, 29), 3, black);
            out.push(DrawCmd::FillRect {
                rect: Rect::new(center_x - 2, top + 3, 4, 29),
                color: black,
            });
        }
        // Library: three book spines with intentionally different heights.
        1 => {
            outline_rect(out, Rect::new(center_x - 22, top + 5, 11, 25), 3, black);
            outline_rect(out, Rect::new(center_x - 5, top, 11, 30), 3, black);
            outline_rect(out, Rect::new(center_x + 12, top + 3, 11, 27), 3, black);
        }
        // Account: compact head-and-shoulders silhouette.
        _ => {
            out.push(DrawCmd::FillRect {
                rect: Rect::new(center_x - 6, top, 12, 4),
                color: black,
            });
            out.push(DrawCmd::FillRect {
                rect: Rect::new(center_x - 10, top + 4, 20, 9),
                color: black,
            });
            out.push(DrawCmd::FillRect {
                rect: Rect::new(center_x - 7, top + 13, 14, 4),
                color: black,
            });
            outline_rect(out, Rect::new(center_x - 20, top + 21, 40, 12), 3, black);
        }
    }
}

fn outline_rect(out: &mut Vec<DrawCmd>, rect: Rect, width: i32, color: Color) {
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

/// Equal-width, centered bottom navigation tabs with one selected item.
#[derive(Clone)]
pub struct TabBar(pub(crate) Rc<RefCell<TabBarState>>);

impl TabBar {
    pub fn new(labels: impl IntoIterator<Item = impl Into<String>>, active: usize) -> Self {
        let labels: Vec<String> = labels.into_iter().map(Into::into).collect();
        Self(Rc::new(RefCell::new(TabBarState {
            common: Common::new(),
            active: active.min(labels.len().saturating_sub(1)),
            labels,
            on_select: None,
        })))
    }

    pub fn on_select(self, handler: impl Fn(usize) + 'static) -> Self {
        self.0.borrow_mut().on_select = Some(Rc::new(handler));
        self
    }

    /// Selects a tab and marks the bar dirty when the indicator moves.
    pub fn set_active(&self, active: usize) {
        let mut state = self.0.borrow_mut();
        let active = active.min(state.labels.len().saturating_sub(1));
        if state.active != active {
            state.active = active;
            state.common.dirty = true;
        }
    }

    pub fn active(&self) -> usize {
        self.0.borrow().active
    }

    pub(crate) fn fire_tap_at(&self, point: Point) -> bool {
        let mut state = self.0.borrow_mut();
        if state.labels.is_empty() || state.common.frame.w <= 0 {
            return false;
        }
        let relative_x = (point.x - state.common.frame.x).clamp(0, state.common.frame.w - 1);
        let index = (relative_x as usize * state.labels.len()) / state.common.frame.w as usize;
        if state.active != index {
            state.active = index;
            state.common.dirty = true;
        }
        let handler = state.on_select.clone();
        drop(state);
        if let Some(handler) = handler {
            handler(index);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn fills_available_width_at_a_fixed_touch_height() {
        let bar = TabBar::new(["Reading", "Library", "Account"], 0);
        assert_eq!(
            bar.0.borrow().measure(Size::new(900, 300)),
            Size::new(900, HEIGHT)
        );
    }

    #[test]
    fn maps_taps_to_equal_width_tabs() {
        let selected = Rc::new(Cell::new(usize::MAX));
        let output = selected.clone();
        let bar = TabBar::new(["Reading", "Library", "Account"], 0).on_select(move |index| {
            output.set(index);
        });
        bar.0.borrow_mut().common.frame = Rect::new(30, 100, 900, HEIGHT);

        assert!(bar.fire_tap_at(Point::new(480, 140)));
        assert_eq!(selected.get(), 1);
        assert_eq!(bar.active(), 1);
        assert!(bar.0.borrow().common.dirty);
    }
}
