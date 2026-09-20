#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    eprintln!("genji_gui is available on Windows only");
}

#[cfg(windows)]
mod app {
    use genji_unpack::localization::{extract_localization, inject_localization};
    use genji_unpack::{rebuild_fdi, unpack};
    use std::mem::size_of;
    use std::path::{Path, PathBuf};
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::Graphics::Gdi::{UpdateWindow, COLOR_WINDOW};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::Controls::Dialogs::{
        GetOpenFileNameW, OFN_FILEMUSTEXIST, OFN_PATHMUSTEXIST, OPENFILENAMEW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    const ID_BROWSE: usize = 100;
    const ID_UNPACK: usize = 101;
    const ID_EXTRACT: usize = 102;
    const ID_INJECT: usize = 103;
    const ID_REBUILD: usize = 104;

    static mut SOURCE_EDIT: HWND = null_mut();
    static mut UNPACK_EDIT: HWND = null_mut();
    static mut WORKSPACE_EDIT: HWND = null_mut();
    static mut INJECT_EDIT: HWND = null_mut();
    static mut REBUILD_EDIT: HWND = null_mut();
    static mut STATUS: HWND = null_mut();

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(Some(0)).collect()
    }

    unsafe fn create_control(
        class: &str,
        text: &str,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: HWND,
        id: usize,
    ) -> HWND {
        let class = wide(class);
        let text = wide(text);
        CreateWindowExW(
            0,
            class.as_ptr(),
            text.as_ptr(),
            style,
            x,
            y,
            width,
            height,
            parent,
            id as *mut core::ffi::c_void,
            GetModuleHandleW(null()),
            null(),
        )
    }

    unsafe fn set_text(hwnd: HWND, value: &str) {
        let value = wide(value);
        SetWindowTextW(hwnd, value.as_ptr());
    }

    unsafe fn get_text(hwnd: HWND) -> String {
        let length = GetWindowTextLengthW(hwnd).max(0) as usize;
        let mut buffer = vec![0u16; length + 1];
        GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
        String::from_utf16_lossy(&buffer[..length])
            .trim()
            .to_owned()
    }

    unsafe fn message(owner: HWND, title: &str, text: &str, error: bool) {
        let title = wide(title);
        let text = wide(text);
        MessageBoxW(
            owner,
            text.as_ptr(),
            title.as_ptr(),
            MB_OK
                | if error {
                    MB_ICONERROR
                } else {
                    MB_ICONINFORMATION
                },
        );
    }

    unsafe fn allow_overwrite(owner: HWND, path: &Path) -> bool {
        if !path.exists() {
            return true;
        }
        let title = wide("确认覆盖");
        let text = wide(&format!("目标已经存在：\n{}\n\n继续写入？", path.display()));
        MessageBoxW(
            owner,
            text.as_ptr(),
            title.as_ptr(),
            MB_YESNO | MB_ICONQUESTION,
        ) == IDYES
    }

    unsafe fn choose_fdi(owner: HWND) -> Option<PathBuf> {
        let mut buffer = [0u16; 32768];
        let title = wide("选择源氏 FDI 镜像");
        let filter = wide("FDI 镜像 (*.FDI)\0*.FDI\0所有文件 (*.*)\0*.*\0");
        let mut dialog = OPENFILENAMEW::default();
        dialog.lStructSize = size_of::<OPENFILENAMEW>() as u32;
        dialog.hwndOwner = owner;
        dialog.lpstrFilter = filter.as_ptr();
        dialog.lpstrFile = buffer.as_mut_ptr();
        dialog.nMaxFile = buffer.len() as u32;
        dialog.lpstrTitle = title.as_ptr();
        dialog.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST;
        if GetOpenFileNameW(&mut dialog) == 0 {
            return None;
        }
        let length = buffer.iter().position(|value| *value == 0).unwrap_or(0);
        Some(PathBuf::from(String::from_utf16_lossy(&buffer[..length])))
    }

    unsafe fn fill_defaults(source: &Path) {
        set_text(SOURCE_EDIT, &source.display().to_string());
        let parent = source.parent().unwrap_or_else(|| Path::new("."));
        let root = parent.join("work");
        set_text(UNPACK_EDIT, &root.join("unpacked").display().to_string());
        set_text(
            WORKSPACE_EDIT,
            &root.join("localization").display().to_string(),
        );
        set_text(INJECT_EDIT, &root.join("rebuilt").display().to_string());
        set_text(
            REBUILD_EDIT,
            &root.join("Genji_rebuilt.FDI").display().to_string(),
        );
    }

    unsafe fn required_path(hwnd: HWND, label: &str) -> Result<PathBuf, String> {
        let value = get_text(hwnd);
        if value.is_empty() {
            Err(format!("请填写{label}"))
        } else {
            Ok(PathBuf::from(value))
        }
    }

    unsafe fn perform(owner: HWND, command: usize) {
        set_text(STATUS, "正在处理，请稍候……");
        let result = (|| -> Result<String, String> {
            let source = required_path(SOURCE_EDIT, "源 FDI")?;
            match command {
                ID_UNPACK => {
                    let output = required_path(UNPACK_EDIT, "解包目录")?;
                    if !allow_overwrite(owner, &output) {
                        return Ok("已取消".to_owned());
                    }
                    let report = unpack(&source, &output, output.exists())?;
                    Ok(format!(
                        "解包完成：{} 个盘内文件，{} 个游戏资源\n{}",
                        report.files.len(),
                        report.resource_count,
                        output.display()
                    ))
                }
                ID_EXTRACT => {
                    let output = required_path(WORKSPACE_EDIT, "翻译工作区")?;
                    if !allow_overwrite(owner, &output) {
                        return Ok("已取消".to_owned());
                    }
                    let report = extract_localization(&source, &output, output.exists())?;
                    Ok(format!(
                        "文本提取完成：{} 条文本，{} 个姓名槽，{} 个选项\n{}",
                        report.entries,
                        report.names,
                        report.choices,
                        output.display()
                    ))
                }
                ID_INJECT => {
                    let workspace = required_path(WORKSPACE_EDIT, "翻译工作区")?;
                    let output = required_path(INJECT_EDIT, "注入输出目录")?;
                    if !allow_overwrite(owner, &output) {
                        return Ok("已取消".to_owned());
                    }
                    let report =
                        inject_localization(&source, &workspace, &output, output.exists())?;
                    Ok(format!(
                        "注入完成：改动 {} 条，重建 {} 张文本表，生成 {} 个字槽\nFDI：{}\n字库：{}",
                        report.changed_entries,
                        report.changed_tables,
                        report.redrawn_slots,
                        report.output_fdi.display(),
                        report.output_font.display()
                    ))
                }
                ID_REBUILD => {
                    let unpacked = required_path(UNPACK_EDIT, "解包目录")?;
                    let output = required_path(REBUILD_EDIT, "资源回包 FDI")?;
                    if !allow_overwrite(owner, &output) {
                        return Ok("已取消".to_owned());
                    }
                    let report = rebuild_fdi(&source, &unpacked, &output, output.exists())?;
                    Ok(format!(
                        "FDI 重建完成：{} 个资源有变化，G1.DAT {} 字节\n{}",
                        report.changed_resources, report.g1_size, report.output
                    ))
                }
                _ => return Err("未知操作".to_owned()),
            }
        })();
        match result {
            Ok(text) => {
                set_text(STATUS, &text.replace('\n', "  "));
                message(owner, "源氏工具", &text, false);
            }
            Err(error) => {
                set_text(STATUS, &format!("错误：{error}"));
                message(owner, "处理失败", &error, true);
            }
        }
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_CREATE => {
                let label_style = WS_CHILD | WS_VISIBLE;
                let edit_style = WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL as u32;
                let button_style = WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32;
                create_control("STATIC", "源 FDI", label_style, 20, 20, 100, 24, hwnd, 0);
                SOURCE_EDIT = create_control("EDIT", "", edit_style, 120, 18, 560, 25, hwnd, 1);
                create_control(
                    "BUTTON",
                    "浏览…",
                    button_style,
                    690,
                    17,
                    80,
                    27,
                    hwnd,
                    ID_BROWSE,
                );
                create_control(
                    "STATIC",
                    "原始资源目录",
                    label_style,
                    20,
                    58,
                    100,
                    24,
                    hwnd,
                    0,
                );
                UNPACK_EDIT =
                    create_control("EDIT", "unpacked", edit_style, 120, 56, 650, 25, hwnd, 2);
                create_control(
                    "STATIC",
                    "翻译工作区",
                    label_style,
                    20,
                    96,
                    100,
                    24,
                    hwnd,
                    0,
                );
                WORKSPACE_EDIT = create_control(
                    "EDIT",
                    "localization",
                    edit_style,
                    120,
                    94,
                    650,
                    25,
                    hwnd,
                    3,
                );
                create_control(
                    "STATIC",
                    "注入输出目录",
                    label_style,
                    20,
                    134,
                    100,
                    24,
                    hwnd,
                    0,
                );
                INJECT_EDIT =
                    create_control("EDIT", "rebuilt", edit_style, 120, 132, 650, 25, hwnd, 4);
                create_control(
                    "STATIC",
                    "资源回包 FDI",
                    label_style,
                    20,
                    172,
                    100,
                    24,
                    hwnd,
                    0,
                );
                REBUILD_EDIT = create_control(
                    "EDIT",
                    "Genji_rebuilt.FDI",
                    edit_style,
                    120,
                    170,
                    650,
                    25,
                    hwnd,
                    5,
                );
                create_control(
                    "BUTTON",
                    "1  解包原始资源",
                    button_style,
                    20,
                    220,
                    180,
                    38,
                    hwnd,
                    ID_UNPACK,
                );
                create_control(
                    "BUTTON",
                    "2  提取翻译 JSON",
                    button_style,
                    210,
                    220,
                    180,
                    38,
                    hwnd,
                    ID_EXTRACT,
                );
                create_control(
                    "BUTTON",
                    "3  注入并重建 FDI/字库",
                    button_style,
                    400,
                    220,
                    180,
                    38,
                    hwnd,
                    ID_INJECT,
                );
                create_control(
                    "BUTTON",
                    "4  从资源目录重建 FDI",
                    button_style,
                    590,
                    220,
                    180,
                    38,
                    hwnd,
                    ID_REBUILD,
                );
                STATUS = create_control(
                    "STATIC",
                    "请选择源 FDI。姓名、正文、选项与显式换行会按已确认结构处理。",
                    label_style,
                    20,
                    280,
                    750,
                    48,
                    hwnd,
                    6,
                );
                if let Ok(exe) = std::env::current_exe() {
                    if let Some(work) = exe.parent() {
                        if let Some(root) = work.parent() {
                            let source = root.join("Genji.FDI");
                            if source.is_file() {
                                fill_defaults(&source);
                            }
                        }
                    }
                }
                0
            }
            WM_COMMAND => {
                let id = wparam & 0xFFFF;
                match id {
                    ID_BROWSE => {
                        if let Some(path) = choose_fdi(hwnd) {
                            fill_defaults(&path);
                        }
                    }
                    ID_UNPACK | ID_EXTRACT | ID_INJECT | ID_REBUILD => perform(hwnd, id),
                    _ => {}
                }
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    pub fn run() -> Result<(), String> {
        unsafe {
            let instance = GetModuleHandleW(null());
            let class_name = wide("GenjiLocalizationWindow");
            let title = wide("源氏 FDI 资源与汉化工具");
            let class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(window_proc),
                hInstance: instance,
                hCursor: LoadCursorW(null_mut(), IDC_ARROW),
                hbrBackground: (COLOR_WINDOW as usize + 1) as *mut core::ffi::c_void,
                lpszClassName: class_name.as_ptr(),
                ..Default::default()
            };
            if RegisterClassW(&class) == 0 {
                return Err("无法注册 Windows 窗口类".to_owned());
            }
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                810,
                390,
                null_mut(),
                null_mut(),
                instance,
                null(),
            );
            if hwnd.is_null() {
                return Err("无法创建主窗口".to_owned());
            }
            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        Ok(())
    }
}

#[cfg(windows)]
fn main() {
    if let Err(error) = app::run() {
        let title: Vec<u16> = "源氏工具错误\0".encode_utf16().collect();
        let message: Vec<u16> = format!("{error}\0").encode_utf16().collect();
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
                std::ptr::null_mut(),
                message.as_ptr(),
                title.as_ptr(),
                windows_sys::Win32::UI::WindowsAndMessaging::MB_OK
                    | windows_sys::Win32::UI::WindowsAndMessaging::MB_ICONERROR,
            );
        }
    }
}
