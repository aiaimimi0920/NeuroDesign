//! TOOLINFO 的文字由 Content 持有，更新时先通知系统再释放旧缓冲。
use super::*;
use windows::Win32::UI::Controls::*;
pub(crate) struct Tooltip(HWND);
impl Tooltip {
    pub(crate) unsafe fn new(owner: HWND) -> Result<Self> {
        InitCommonControlsEx(&INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_WIN95_CLASSES,
        })
        .ok()?;
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            TOOLTIPS_CLASSW,
            None,
            WS_POPUP | WINDOW_STYLE(TTS_ALWAYSTIP | TTS_NOPREFIX),
            0,
            0,
            0,
            0,
            Some(owner),
            None,
            None,
            None,
        )?;
        SendMessageW(hwnd, TTM_SETTIPBKCOLOR, Some(WPARAM(0x18120e)), None);
        SendMessageW(hwnd, TTM_SETTIPTEXTCOLOR, Some(WPARAM(0xeff8f7)), None);
        Ok(Self(hwnd))
    }
    unsafe fn info(owner: HWND, child: HWND, text: &HSTRING) -> TTTOOLINFOW {
        TTTOOLINFOW {
            // 独立 crate/Gallery 不强制宿主启用 V6 manifest；V2 布局不含 V6 的 lpReserved。
            cbSize: std::mem::offset_of!(TTTOOLINFOW, lpReserved) as u32,
            uFlags: TTF_IDISHWND | TTF_SUBCLASS,
            hwnd: owner,
            uId: child.0 as usize,
            lpszText: windows::core::PWSTR(text.as_ptr() as *mut _),
            ..Default::default()
        }
    }
    pub(crate) unsafe fn add(&self, owner: HWND, child: HWND, text: &HSTRING) -> Result<()> {
        let info = Self::info(owner, child, text);
        if SendMessageW(
            self.0,
            TTM_ADDTOOLW,
            None,
            Some(LPARAM((&info as *const TTTOOLINFOW) as isize)),
        )
        .0 == 0
        {
            return Err(Error::new(E_FAIL, "Could not register toolbar tooltip"));
        }
        Ok(())
    }
    pub(crate) unsafe fn remove(&self, owner: HWND, child: HWND, text: &HSTRING) {
        let info = Self::info(owner, child, text);
        SendMessageW(
            self.0,
            TTM_DELTOOLW,
            None,
            Some(LPARAM((&info as *const TTTOOLINFOW) as isize)),
        );
    }
    pub(crate) unsafe fn update(&self, owner: HWND, child: HWND, text: &HSTRING) {
        let info = Self::info(owner, child, text);
        SendMessageW(
            self.0,
            TTM_UPDATETIPTEXTW,
            None,
            Some(LPARAM((&info as *const TTTOOLINFOW) as isize)),
        );
    }
}
impl Drop for Tooltip {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}
