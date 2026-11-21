//! SV/色相/alpha 的单次捕获及调色板操作；拖动只更改弹层草稿，绝不提前提交应用颜色。
use super::*;
pub(super) unsafe fn point(hwnd: HWND, d: &Data, mut screen: POINT) -> Option<POINT> {
    if !ScreenToClient(hwnd, &mut screen).as_bool() {
        return None;
    }
    Some(POINT {
        x: ((screen.x + d.origin.get().x) as f32 * 96.0 / d.dpi as f32).floor() as i32,
        y: ((screen.y + d.origin.get().y) as f32 * 96.0 / d.dpi as f32).floor() as i32,
    })
}
pub(in super::super) unsafe fn sample(hwnd: HWND, d: &Data, p: POINT) {
    let mut v = d.value.get();
    let norm = |n: i32, a: i32, b: i32| ((n - a) as f32 / (b - a) as f32).clamp(0.0, 1.0);
    match d.drag.get() {
        Some(0) => {
            v.s = norm(p.x, geometry::SV.left, geometry::SV.right);
            v.v = 1.0 - norm(p.y, geometry::SV.top, geometry::SV.bottom);
        }
        Some(1) => v.h = norm(p.x, geometry::HUE.left, geometry::HUE.right) * 360.0,
        Some(2) => v.a = norm(p.x, geometry::ALPHA.left, geometry::ALPHA.right),
        _ => return,
    }
    d.hex_editing.set(false);
    d.value.set(v);
    controls::refresh(hwnd, d);
}
pub(super) unsafe fn palette_changed(hwnd: HWND, d: &Data) {
    d.scroll.set(
        d.scroll
            .get()
            .min(geometry::max_scroll(d.palette.borrow().len())),
    );
    let _ = InvalidateRect(Some(controls::child(hwnd, controls::PALETTE)), None, false);
}
pub(in super::super) fn add(d: &Data) {
    let color = d.value.get().color();
    let mut palette = d.palette.borrow_mut();
    if palette.len() < 4096 && !palette.iter().any(|c| c.same_palette(color)) {
        palette.insert(0, color);
        d.selected.set(None);
    }
}
pub(in super::super) fn remove(d: &Data, index: usize) {
    let mut palette = d.palette.borrow_mut();
    if index < palette.len() {
        let color = palette.remove(index);
        let mut removed = d.removed.borrow_mut();
        if !removed.iter().any(|c| c.same_palette(color)) {
            if removed.len() == 4096 {
                *d.error.borrow_mut() = Some(Error::new(E_BOUNDS, "Too many palette removals"));
                d.closed.set(true);
            } else {
                removed.push(color);
            }
        }
        d.selected.set(None);
    }
}
pub(in super::super) unsafe fn palette_click(hwnd: HWND, d: &Data, p: POINT, right: bool) {
    let count = d.palette.borrow().len();
    for i in 0..=count {
        let r = geometry::swatch(i, d.scroll.get());
        let remove_hit = geometry::rect(r.right - 6, r.top - 5, 13, 13);
        if i > 0 && (geometry::inside(remove_hit, p) || right && geometry::inside(r, p)) {
            remove(d, i - 1);
            break;
        }
        if geometry::inside(r, p) {
            if i == 0 {
                add(d);
            } else {
                let color = d.palette.borrow()[i - 1];
                let mut v = d.value.get();
                if color.a == 0 {
                    v.a = 0.0;
                } else {
                    v.load(color);
                }
                d.value.set(v);
                d.hex_editing.set(false);
                d.selected.set(Some(i - 1));
            }
            break;
        }
    }
    controls::refresh(hwnd, d);
    palette_changed(hwnd, d);
}
pub(in super::super) unsafe fn down(hwnd: HWND, d: &Data, p: POINT, right: bool) -> bool {
    if geometry::inside(geometry::PALETTE, p) {
        let _ = SetFocus(Some(controls::child(hwnd, controls::PALETTE)));
        palette_click(hwnd, d, p, right);
        return true;
    }
    if !right {
        for (i, r) in [geometry::SV, geometry::HUE, geometry::ALPHA]
            .into_iter()
            .enumerate()
        {
            if geometry::inside(r, p) {
                let _ = SetFocus(Some(hwnd));
                d.drag.set(Some(i as u8));
                SetCapture(hwnd);
                sample(hwnd, d, p);
                return true;
            }
        }
    }
    false
}
