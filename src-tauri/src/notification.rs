use windows::core::HSTRING;
use windows::Win32::UI::WindowsAndMessaging::{MB_OK, MB_SETFOREGROUND, MB_SYSTEMMODAL, MessageBoxW};

/// 显示 Windows 原生通知
///
/// 当前使用 MessageBoxW 实现，后续可升级为 WinRT Toast 通知：
/// ```rust
/// // TODO: WinRT Toast 通知方案（需 AppUserModelId 注册）
/// // use windows::UI::Notifications::ToastNotificationManager;
/// // let notifier = ToastNotificationManager::CreateToastNotifierWithId("com.toumato.app")?;
/// // notifier.Show(&notification)?;
/// ```
pub fn show_notification(title: &str, body: &str) -> Result<(), String> {
    let title_hstring = HSTRING::from(title);
    let body_hstring = HSTRING::from(body);

    let result = unsafe {
        MessageBoxW(
            None,
            &body_hstring,
            &title_hstring,
            MB_OK | MB_SETFOREGROUND | MB_SYSTEMMODAL,
        )
    };

    if result.0 == 0 {
        Err("显示通知失败".to_string())
    } else {
        Ok(())
    }
}
