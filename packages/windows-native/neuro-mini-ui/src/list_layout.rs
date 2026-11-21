//! 主干长下拉的固定字号/行高、屏幕约束和像素滚动几何。
use super::*;
#[derive(Clone, Copy)]
pub(super) struct Layout {
    pub rect: RECT,
    pub viewport: RECT,
    pub row: i32,
    pub pad: i32,
    pub dpi: u32,
    pub total: i32,
}
pub(super) fn inside(r: RECT, p: POINT) -> bool {
    p.x >= r.left && p.x < r.right && p.y >= r.top && p.y < r.bottom
}
impl Layout {
    pub fn new(anchor: RECT, work: RECT, dpi: u32, count: usize) -> Result<Self> {
        let px = |n| crate::layout::px(n, dpi);
        let border = px(1);
        let row = px(28);
        let pad = px(4);
        let total = row * count as i32 + pad * 2;
        let width = px(196).min(work.right - work.left - px(8));
        let height = (total.min(px(220)) + border * 2).min(work.bottom - work.top - px(8));
        if width < px(32) || height < row + border * 2 {
            return Err(Error::new(
                E_BOUNDS,
                "Scrolling list does not fit work area",
            ));
        }
        let left = anchor
            .left
            .clamp(work.left + px(4), work.right - px(4) - width);
        let top = if anchor.bottom + px(4) + height <= work.bottom - px(4) {
            anchor.bottom + px(4)
        } else {
            anchor.top - px(4) - height
        }
        .clamp(work.top + px(4), work.bottom - px(4) - height);
        let viewport = RECT {
            left: border,
            top: border,
            right: width
                - border
                - if total > height - border * 2 {
                    px(12)
                } else {
                    0
                },
            bottom: height - border,
        };
        Ok(Self {
            rect: RECT {
                left,
                top,
                right: left + width,
                bottom: top + height,
            },
            viewport,
            row,
            pad,
            dpi,
            total,
        })
    }
    pub fn height(self) -> i32 {
        self.viewport.bottom - self.viewport.top
    }
    pub fn maximum(self) -> f64 {
        (self.total - self.height()).max(0) as f64
    }
    pub fn track(self) -> RECT {
        let right = self.rect.right - self.rect.left - crate::layout::px(4, self.dpi);
        RECT {
            left: right - crate::layout::px(8, self.dpi),
            top: self.viewport.top,
            right,
            bottom: self.viewport.bottom,
        }
    }
    pub fn thumb(self, top: f64) -> RECT {
        let r = self.track();
        let height =
            self.height().min(crate::layout::px(18, self.dpi).max(
                (self.height() as f64 * self.height() as f64 / self.total as f64).round() as i32,
            ));
        let y = if self.maximum() == 0.0 {
            0
        } else {
            (top / self.maximum() * (self.height() - height) as f64).round() as i32
        };
        RECT {
            top: r.top + y,
            bottom: r.top + y + height,
            ..r
        }
    }
    pub fn hit(self, p: POINT, top: f64, count: usize) -> Option<usize> {
        if !inside(self.viewport, p) {
            return None;
        }
        let y = p.y - self.viewport.top + top.round() as i32 - self.pad;
        (y >= 0)
            .then_some((y / self.row) as usize)
            .filter(|i| *i < count)
    }
}
