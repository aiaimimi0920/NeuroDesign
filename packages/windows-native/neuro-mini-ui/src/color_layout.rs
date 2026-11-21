//! 固定 248 DIP 面板和八列调色板；屏幕物理坐标只在此转换，列表在自身 64 DIP 区域滚动。
use windows::Win32::Foundation::{POINT, RECT};
pub(super) const WIDTH: i32 = 248;
pub(super) const HEIGHT: i32 = 318;
pub(super) fn rect(x: i32, y: i32, w: i32, h: i32) -> RECT {
    RECT {
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
    }
}
pub(super) fn inside(r: RECT, p: POINT) -> bool {
    p.x >= r.left && p.x < r.right && p.y >= r.top && p.y < r.bottom
}
pub(super) const SV: RECT = RECT {
    left: 9,
    top: 9,
    right: 239,
    bottom: 117,
};
pub(super) const HUE: RECT = RECT {
    left: 9,
    top: 125,
    right: 239,
    bottom: 137,
};
pub(super) const ALPHA: RECT = RECT {
    left: 9,
    top: 145,
    right: 239,
    bottom: 157,
};
pub(super) const PALETTE: RECT = RECT {
    left: 9,
    top: 205,
    right: 239,
    bottom: 269,
};
pub(super) fn swatch(index: usize, scroll: i32) -> RECT {
    rect(
        9 + (index % 8) as i32 * 29,
        212 + (index / 8) as i32 * 31 - scroll,
        23,
        22,
    )
}
pub(super) fn max_scroll(count: usize) -> i32 {
    (((count + 1).div_ceil(8) as i32 * 31 + 11) - 64).max(0)
}
pub(super) fn anchored(anchor: RECT, work: RECT, dpi: u32) -> RECT {
    let px = |v| crate::layout::px(v, dpi);
    let w = px(WIDTH).min((work.right - work.left - px(16)).max(1));
    let h = px(HEIGHT).min((work.bottom - work.top - px(16)).max(1));
    let mut x = anchor.right + px(8);
    let mut y = anchor.top;
    if x + w > work.right - px(8) {
        x = anchor.left - w - px(8);
    }
    if x < work.left + px(8) {
        x = (anchor.left + anchor.right - w) / 2;
        y = if anchor.bottom + px(8) + h < work.bottom {
            anchor.bottom + px(8)
        } else {
            anchor.top - h - px(8)
        };
    }
    x = x.clamp(
        work.left + px(8),
        (work.right - w - px(8)).max(work.left + px(8)),
    );
    y = y.clamp(
        work.top + px(8),
        (work.bottom - h - px(8)).max(work.top + px(8)),
    );
    RECT {
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
    }
}
