//! 一个附属裁剪视口承载通知；应用持有条目和业务，组件只拥有几何、输入与 HWND。
use super::*;
use std::cell::RefCell;
use windows::Win32::UI::{Input::KeyboardAndMouse::GetFocus, Shell::*};
#[path = "notice_layout.rs"]
mod layout;
const CLASS: windows::core::PCWSTR = w!("NeuroMini.Native.NoticeStack");
pub(super) struct Data {
    pub(super) root: Cell<HWND>,
    parent: HWND,
    pub(super) entries: RefCell<Vec<Weak<Cell<HWND>>>>,
    dpi: Cell<u32>,
    affinity: Cell<u32>,
    scroll: scroll::Scroll,
    placing: Cell<bool>,
    region: RefCell<Vec<RECT>>,
    frame: Cell<(i32, i32, i32, u64, u32)>,
}
pub struct NoticeStack {
    data: Rc<Data>,
}
impl NoticeStack {
    /// # Safety
    /// owner 为当前 UI 线程有效顶层 HWND；所有方法及 Drop 必须在该线程调用。
    pub unsafe fn new(owner: HWND) -> Result<Self> {
        if !IsWindow(Some(owner)).as_bool() || GetAncestor(owner, GA_ROOT) != owner {
            return Err(Error::new(E_INVALIDARG, "Invalid notice owner"));
        }
        let instance = HINSTANCE(GetModuleHandleW(None)?.0);
        register(CLASS, procedure, instance, IDC_ARROW)?;
        let data = Rc::new(Data {
            root: Cell::default(),
            parent: owner,
            entries: RefCell::default(),
            dpi: Cell::new(96),
            affinity: Cell::new(0),
            scroll: scroll::Scroll::default(),
            placing: Cell::new(false),
            region: RefCell::default(),
            frame: Cell::default(),
        });
        let claimed = Rc::new(Cell::new(false));
        let pointer = Box::into_raw(Box::new((data.clone(), claimed.clone())));
        let result = CreateWindowExW(
            WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_CONTROLPARENT,
            CLASS,
            w!("通知"),
            WS_POPUP | WS_CLIPCHILDREN,
            0,
            0,
            1,
            1,
            Some(owner),
            None,
            Some(instance),
            Some(pointer.cast()),
        );
        let root = match result {
            Ok(root) => root,
            Err(error) => {
                if !claimed.get() {
                    drop(Box::from_raw(pointer));
                }
                return Err(error);
            }
        };
        let stack = Self { data };
        if !SetWindowSubclass(owner, Some(follow), root.0 as usize, root.0 as usize).as_bool() {
            return Err(Error::from_win32());
        }
        stack.place()?;
        Ok(stack)
    }
    pub fn window(&self) -> Option<HWND> {
        let hwnd = self.data.root.get();
        (!hwnd.is_invalid()).then_some(hwnd)
    }
    /// # Safety
    /// 同 new；最新项置顶。调用方拥有 Notice，其 Drop、点击或五秒超时立即回收并重排。
    pub unsafe fn push(&self, title: &str, message: &str, dismiss: &str) -> Result<Notice> {
        Notice::new(&self.data, title, message, dismiss)
    }
    /// # Safety
    /// 同 new；同步 owner 的显隐、大小、DPI 和捕获排除，不改变 owner 层序。
    pub unsafe fn place(&self) -> Result<()> {
        layout::place(&self.data)
    }
    /// # Safety
    /// 同 new；仅处理本视口及其子控件的 Tab/键盘消息，不截获宿主输入。
    pub unsafe fn translate(&self, message: &MSG) -> bool {
        let Some(root) = self.window() else {
            return false;
        };
        if !(message.hwnd == root || IsChild(root, message.hwnd).as_bool()) {
            return false;
        }
        if message.message == WM_KEYDOWN && message.wParam.0 == 9 {
            let result = IsDialogMessageW(root, message).as_bool();
            layout::reveal(&self.data, GetFocus());
            return result;
        }
        false
    }
}
impl Drop for NoticeStack {
    fn drop(&mut self) {
        unsafe {
            if let Some(root) = self.window() {
                let _ = DestroyWindow(root);
            }
        }
    }
}
pub(super) unsafe fn changed(data: &Data) -> Result<()> {
    data.scroll.cancel(data.root.get());
    data.scroll.top.set(0.0);
    layout::place(data)
}
unsafe extern "system" fn follow(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
    id: usize,
    root: usize,
) -> LRESULT {
    // 默认过程可同步销毁 owner/root；先取得 Rc，之后不再沿可能复用的 HWND 取裸指针。
    let root = HWND(root as *mut _);
    let pointer = GetWindowLongPtrW(root, GWLP_USERDATA) as *const (Rc<Data>, Rc<Cell<bool>>);
    let data = (!pointer.is_null()).then(|| (*pointer).0.clone());
    let result = DefSubclassProc(hwnd, msg, wp, lp);
    if msg == WM_NCDESTROY {
        let _ = RemoveWindowSubclass(hwnd, Some(follow), id);
    } else if matches!(msg, WM_WINDOWPOSCHANGED | WM_SHOWWINDOW | WM_DPICHANGED) {
        if let Some(data) = data.filter(|data| data.root.get() == root) {
            if let Err(error) = layout::place(&data) {
                fail(root, error);
            }
        }
    }
    result
}
pub(super) unsafe fn fail(root: HWND, error: Error) {
    // 不保留布局/保护失败后的旧像素；诊断交给应用日志，不伪装成功。
    let _ = ShowWindow(root, SW_HIDE);
    eprintln!("neuro-mini-notice: {error}");
}
unsafe extern "system" fn procedure(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        let pointer =
            (*(lp.0 as *const CREATESTRUCTW)).lpCreateParams as *mut (Rc<Data>, Rc<Cell<bool>>);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, pointer as isize);
        let (data, claimed) = &*pointer;
        claimed.set(true);
        data.root.set(hwnd);
        return LRESULT(1);
    }
    let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut (Rc<Data>, Rc<Cell<bool>>);
    if pointer.is_null() {
        return DefWindowProcW(hwnd, msg, wp, lp);
    }
    let data = &(*pointer).0;
    if data.scroll.input(hwnd, data.dpi.get(), msg, wp, lp) {
        if let Err(error) = layout::place(data) {
            fail(hwnd, error);
        }
        return LRESULT(0);
    }
    match msg {
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_MOUSEWHEEL => LRESULT(0),
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let dc = BeginPaint(hwnd, &mut ps);
            let brush = CreateSolidBrush(paint::color(crate::Palette::default().control));
            FillRect(dc, &ps.rcPaint, brush);
            let _ = DeleteObject(HGDIOBJ(brush.0));
            data.scroll.paint(dc, data.dpi.get());
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_DESTROY => {
            data.root.set(HWND::default());
            data.scroll.cancel(hwnd);
            let _ = RemoveWindowSubclass(data.parent, Some(follow), hwnd.0 as usize);
            LRESULT(0)
        }
        WM_NCDESTROY => {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            drop(Box::from_raw(pointer));
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}
