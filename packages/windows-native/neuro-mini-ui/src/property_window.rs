//! 属性条附着在主工具条外壳上方，空间不足向下；不显示独立标题或任务栏项。
use super::*;
pub(super) unsafe fn create(toolbar: HWND) -> Result<Bar> {
    create_at(toolbar, None)
}
pub(super) unsafe fn create_at(toolbar: HWND, stack_width: Option<u16>) -> Result<Bar> {
    let class = w!("NeuroMini.Native.PropertyBar");
    if RegisterClassW(&WNDCLASSW {
        lpszClassName: class,
        lpfnWndProc: Some(procedure),
        hCursor: LoadCursorW(None, IDC_ARROW)?,
        ..Default::default()
    }) == 0
        && GetLastError() != ERROR_CLASS_ALREADY_EXISTS
    {
        return Err(Error::from_win32());
    }
    let dpi = GetDpiForWindow(toolbar).max(96);
    let painter = Painter::new()?;
    let font = controls::font(dpi, 12)?;
    let small_font = match controls::font(dpi, 10) {
        Ok(f) => f,
        Err(e) => {
            let _ = DeleteObject(HGDIOBJ(font.0));
            return Err(e);
        }
    };
    let brush = CreateSolidBrush(paint::rgb(crate::paint::mix(
        painter.palette.control,
        painter.palette.text,
        0.06,
    )));
    if brush.is_invalid() {
        let _ = DeleteObject(HGDIOBJ(font.0));
        let _ = DeleteObject(HGDIOBJ(small_font.0));
        return Err(Error::from_win32());
    }
    let data = Box::new(Data {
        toolbar,
        stack_width,
        context: Cell::new(0),
        tip: RefCell::new(None),
        controls: RefCell::new(Vec::new()),
        section_starts: RefCell::new(Vec::new()),
        active: Cell::new(false),
        updating: Cell::new(false),
        painter,
        join: Cell::new(crate::strip_shell::Join::default()),
        font: Cell::new(font),
        small_font: Cell::new(small_font),
        brush,
        dpi: Cell::new(dpi),
        available: Cell::new(0),
        width: Cell::new(1),
        height: Cell::new(38),
        viewport: Cell::new(HWND::default()),
        max_height: Cell::new(i32::MAX),
        available_height: Cell::new(i32::MAX),
        content_height: Cell::new(0),
        scroll_top: Cell::new(0.0),
        scroll_drag: Cell::new(None),
    });
    let hwnd = match CreateWindowExW(
        WS_EX_TOOLWINDOW,
        class,
        w!("工具属性"),
        WS_POPUP,
        0,
        0,
        1,
        38,
        Some(toolbar),
        None,
        None,
        None,
    ) {
        Ok(h) => h,
        Err(e) => {
            let _ = DeleteObject(HGDIOBJ(font.0));
            let _ = DeleteObject(HGDIOBJ(small_font.0));
            let _ = DeleteObject(HGDIOBJ(brush.0));
            return Err(e);
        }
    };
    let bar = Bar { hwnd, data };
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, &*bar.data as *const Data as isize);
    if stack_width.is_some() {
        let viewport = CreateWindowExW(
            WS_EX_CONTROLPARENT,
            class,
            w!(""),
            WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN,
            0,
            0,
            1,
            1,
            Some(hwnd),
            None,
            None,
            None,
        )?;
        bar.data.viewport.set(viewport);
        SetWindowLongPtrW(viewport, GWLP_USERDATA, &*bar.data as *const Data as isize);
    }
    // 附属窗口首次显示前继承主条的截图排除；后续切换由宿主一起管理。
    let mut affinity = 0;
    GetWindowDisplayAffinity(toolbar, &mut affinity)?;
    SetWindowDisplayAffinity(hwnd, WINDOW_DISPLAY_AFFINITY(affinity))?;
    *bar.data.tip.borrow_mut() = Some(toolbar::tooltip::Tooltip::new(hwnd)?);
    Ok(bar)
}
unsafe extern "system" fn procedure(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    std::panic::catch_unwind(|| route(hwnd, msg, wp, lp)).unwrap_or(LRESULT(0))
}
unsafe fn route(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let Some(d) = (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Data).as_ref() else {
        return DefWindowProcW(hwnd, msg, wp, lp);
    };
    match scroll::handle(hwnd, d, msg, wp, lp) {
        Ok(Some(value)) => return value,
        Ok(None) => (),
        Err(error) => {
            eprintln!("neuro-mini-properties-scroll: {error}");
            return LRESULT(0);
        }
    }
    match msg {
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let dc = BeginPaint(hwnd, &mut ps);
            if let Err(e) = crate::paint_batch::window(hwnd, dc, |out| paint::shell(d, hwnd, out)) {
                eprintln!("neuro-mini-properties-paint: {e}");
            }
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_PRINTCLIENT => {
            if let Err(e) = paint::shell(d, hwnd, HDC(wp.0 as *mut _)) {
                eprintln!("neuro-mini-properties-print: {e}");
            }
            LRESULT(0)
        }
        WM_WINDOWPOSCHANGED
            if lp.0 != 0 && (*(lp.0 as *const WINDOWPOS)).flags.contains(SWP_SHOWWINDOW) =>
        {
            let result = DefWindowProcW(hwnd, msg, wp, lp);
            if let Err(error) = crate::paint_batch::present(hwnd) {
                eprintln!("neuro-mini-properties-show-paint: {error}");
            }
            result
        }
        WM_CTLCOLOREDIT | WM_CTLCOLORBTN => {
            let dc = HDC(wp.0 as *mut _);
            SetBkColor(
                dc,
                paint::rgb(crate::paint::mix(
                    d.painter.palette.control,
                    d.painter.palette.text,
                    0.06,
                )),
            );
            let active_ratio = msg == WM_CTLCOLOREDIT
                && d.controls.try_borrow().ok().is_some_and(|fields| {
                    fields.iter().any(|c| {
                        matches!(c.field.kind, PropertyKind::Ratio { active: true, .. })
                            && c.ratio
                                .as_ref()
                                .is_some_and(|r| r.values.contains(&HWND(lp.0 as *mut _)))
                    })
                });
            SetTextColor(
                dc,
                paint::rgb(if active_ratio {
                    d.painter.palette.signal
                } else {
                    d.painter.palette.text
                }),
            );
            LRESULT(d.brush.0 as isize)
        }
        WM_CTLCOLORSTATIC => {
            let dc = HDC(wp.0 as *mut _);
            SetBkColor(
                dc,
                paint::rgb(crate::paint::mix(
                    d.painter.palette.control,
                    d.painter.palette.text,
                    0.06,
                )),
            );
            SetTextColor(dc, paint::rgb(d.painter.palette.muted));
            LRESULT(d.brush.0 as isize)
        }
        WM_DRAWITEM if lp.0 != 0 => {
            let draw = &*(lp.0 as *const DRAWITEMSTRUCT);
            if let Err(e) = crate::paint_batch::control(draw.hDC, draw.rcItem, |out| {
                let mut buffered = *draw;
                buffered.hDC = out;
                paint::control(d, &buffered)
            }) {
                eprintln!("neuro-mini-properties-control: {e}");
            }
            LRESULT(1)
        }
        WM_COMMAND => {
            if d.updating.get() {
                return LRESULT(0);
            }
            let id = (wp.0 & 0xffff) as u16 / 4;
            let part = wp.0 & 3;
            let code = wp.0 >> 16;
            let child = HWND(lp.0 as *mut _);
            if ratio::command(hwnd, d, child, code) {
                return LRESULT(0);
            }
            if code == EN_CHANGE as usize {
                if let Some(c) = d.controls.borrow_mut().iter_mut().find(|c| {
                    c.hwnd == child || c.ratio.as_ref().is_some_and(|r| r.values.contains(&child))
                }) {
                    c.dirty = true;
                }
            } else if code == BN_CLICKED as usize {
                if part > 0 {
                    let field = d
                        .controls
                        .borrow()
                        .iter()
                        .find(|c| c.field.id == id)
                        .map(|c| c.hwnd);
                    if let Some(field) = field {
                        edit::commit(hwnd, field, if part == 1 { 1 } else { -1 }, false);
                    }
                } else {
                    let first = {
                        let mut fields = d.controls.borrow_mut();
                        let Some(c) = fields.iter_mut().find(|c| c.hwnd == child) else {
                            return LRESULT(0);
                        };
                        if matches!(c.field.kind, PropertyKind::Label { .. }) {
                            return LRESULT(0);
                        }
                        let first = c.pending.is_none();
                        c.pending = Some(PropertyEdit {
                            context: d.context.get(),
                            id,
                            value: None,
                            active: None,
                        });
                        first
                    };
                    if first {
                        edit::notify(d, id);
                    }
                }
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            if d.stack_width.is_some() {
                d.active.set(false);
                let _ = ShowWindow(hwnd, SW_HIDE);
                let _ = SetFocus(Some(d.toolbar));
                if let Some(ui) = toolbar::ui(d.toolbar) {
                    let _ = PostMessageW(Some(ui.owner), ui.notify, WPARAM(0), LPARAM(PANEL_CLOSE));
                }
            } else {
                SendMessageW(d.toolbar, WM_CLOSE, None, None);
            }
            LRESULT(0)
        }
        WM_NCDESTROY => {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}
