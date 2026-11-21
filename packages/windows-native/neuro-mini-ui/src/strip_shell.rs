//! 相邻工具行共用笔直接边；绘制与 HWND region 消费同一接触区间，不改变控件命中或布局。
use windows::{
    core::{Error, Result},
    Win32::{
        Foundation::*,
        Graphics::{Direct2D::*, Gdi::*},
    },
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Join {
    top: Option<(i32, i32)>,
    bottom: Option<(i32, i32)>,
}
impl Join {
    pub(crate) fn between(rect: RECT, neighbour: RECT) -> Self {
        let range = (
            rect.left.max(neighbour.left) - rect.left,
            rect.right.min(neighbour.right) - rect.left,
        );
        if range.0 >= range.1 {
            return Self::default();
        }
        Self {
            top: (rect.top == neighbour.bottom).then_some(range),
            bottom: (rect.bottom == neighbour.top).then_some(range),
        }
    }
    fn corners(self, width: i32) -> [bool; 4] {
        let left = |edge: Option<(i32, i32)>| edge.is_some_and(|(a, _)| a <= 0);
        let right = |edge: Option<(i32, i32)>| edge.is_some_and(|(_, b)| b >= width);
        [
            left(self.top),
            right(self.top),
            right(self.bottom),
            left(self.bottom),
        ]
    }
}
struct Region(HRGN);
impl Drop for Region {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.0 .0));
        }
    }
}
pub(crate) unsafe fn region(
    hwnd: HWND,
    width: i32,
    height: i32,
    dpi: u32,
    join: Join,
) -> Result<()> {
    let radius = crate::layout::px(9, dpi);
    let base = Region(CreateRoundRectRgn(
        0,
        0,
        width + 1,
        height + 1,
        radius * 2,
        radius * 2,
    ));
    if base.0.is_invalid() {
        return Err(Error::from_win32());
    }
    for (square, (x, y)) in join.corners(width).into_iter().zip([
        (0, 0),
        (width - radius, 0),
        (width - radius, height - radius),
        (0, height - radius),
    ]) {
        if square {
            let corner = Region(CreateRectRgn(x, y, x + radius + 1, y + radius + 1));
            if corner.0.is_invalid()
                || CombineRgn(Some(base.0), Some(base.0), Some(corner.0), RGN_OR) == RGN_ERROR
            {
                return Err(Error::from_win32());
            }
        }
    }
    if SetWindowRgn(hwnd, Some(base.0), false) == 0 {
        return Err(Error::from_win32());
    }
    std::mem::forget(base); // 成功后 region 所有权转移到 HWND。
    let _ = InvalidateRect(Some(hwnd), None, false);
    Ok(())
}
pub(crate) unsafe fn outline(
    factory: &ID2D1Factory,
    width: i32,
    height: i32,
    dpi: u32,
    join: Join,
) -> Result<ID2D1PathGeometry> {
    let s = dpi.max(96) as f32 / 96.0;
    let (l, t, r, b) = (
        0.5 * s,
        0.5 * s,
        width as f32 - 0.5 * s,
        height as f32 - 0.5 * s,
    );
    let radius = 9.0 * s;
    let [tl, tr, br, bl] = join
        .corners(width)
        .map(|square| if square { 0.0 } else { radius });
    let mut path = String::new();
    let mut horizontal = |a: f32, z: f32, y: f32, edge: Option<(i32, i32)>| {
        if let Some((start, end)) = edge {
            let end_left = z.min(start as f32);
            let start_right = a.max(end as f32);
            if end_left > a {
                path += &format!("M{a} {y}H{end_left}");
            }
            if z > start_right {
                path += &format!("M{start_right} {y}H{z}");
            }
        } else {
            path += &format!("M{a} {y}H{z}");
        }
    };
    horizontal(l + tl, r - tr, t, join.top);
    horizontal(l + bl, r - br, b, join.bottom);
    path += &format!("M{l} {}V{}M{r} {}V{}", t + tl, b - bl, t + tr, b - br);
    for (rad, x1, y1, x2, y2) in [
        (tl, l, t + tl, l + tl, t),
        (tr, r - tr, t, r, t + tr),
        (br, r, b - br, r - br, b),
        (bl, l + bl, b, l, b - bl),
    ] {
        if rad > 0.0 {
            path += &format!("M{x1} {y1}A{rad} {rad} 0 0 1 {x2} {y2}");
        }
    }
    crate::path::geometry_at(factory, &path, 1.0, (0.0, 0.0), false)
}

#[cfg(test)]
#[path = "strip_shell_tests.rs"]
mod tests;
