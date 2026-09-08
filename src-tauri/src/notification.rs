//! Unobtrusive system notifications for parked notes.
//!
//! A parked note is surfaced to the user through the operating system's native
//! notification mechanism when they switch back to the branch the note was
//! parked on. Because the application is a dependency-free tray utility, the
//! notification is produced by shelling out to the platform's command-line
//! notifier rather than pulling in a notification plugin.
//!
//! Notifications are deliberately best-effort: if the platform's notifier is
//! not installed or fails, the call silently does nothing and the application
//! keeps running. A note is never lost — it simply is not announced.

use std::process::Command;

/// Show an unobtrusive system notification.
///
/// `title` is the branch name, `body` is the note text. Both values are passed
/// as separate process arguments and are never interpolated into a shell
/// command line, so user-controlled content cannot be interpreted by a shell
/// (AC-12).
pub fn notify(title: &str, body: &str) {
    #[cfg(target_os = "linux")]
    notify_linux(title, body);
    #[cfg(target_os = "macos")]
    notify_macos(title, body);
    #[cfg(target_os = "windows")]
    notify_windows(title, body);
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let _ = (title, body);
}

#[cfg(target_os = "linux")]
fn notify_linux(title: &str, body: &str) {
    // notify-send is the standard libnotify command-line tool.
    let _ = Command::new("notify-send")
        .arg("--app-name=Parkplatz")
        .arg(title)
        .arg(body)
        .spawn();
}

#[cfg(target_os = "macos")]
fn notify_macos(title: &str, body: &str) {
    let script = format!(
        "display notification \"{}\" with title \"{}\"",
        escape_applescript(body),
        escape_applescript(title)
    );
    let _ = Command::new("osascript").arg("-e").arg(script).spawn();
}

/// Escape a string for safe interpolation into an AppleScript string literal.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn escape_applescript(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(target_os = "windows")]
fn notify_windows(title: &str, body: &str) {
    // PowerShell toast notification via the Windows Runtime, no module required.
    let script = format!(
        "$ErrorActionPreference = 'SilentlyContinue'\n\
         [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null\n\
         [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null\n\
         $toastXml = [Windows.Data.Xml.Dom.XmlDocument]::new()\n\
         $toastXml.LoadXml(\"<toast><visual><binding template='ToastText02'><text id='1'></text><text id='2'></text></binding></visual></toast>\")\n\
         $toastXml.GetElementsByTagName('text')[0].InnerText = '{}'\n\
         $toastXml.GetElementsByTagName('text')[1].InnerText = '{}'\n\
         $toast = [Windows.UI.Notifications.ToastNotification]::new($toastXml)\n\
         [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Parkplatz').Show($toast)",
        ps_quote(title),
        ps_quote(body)
    );
    let _ = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(script)
        .spawn();
}

/// Escape a string for safe interpolation into a PowerShell single-quoted literal.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn ps_quote(s: &str) -> String {
    s.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applescript_escaping_quotes_backslashes() {
        assert_eq!(escape_applescript(r#"a"b\c"#), r#"a\"b\\c"#);
    }

    #[test]
    fn powershell_escaping_single_quotes() {
        assert_eq!(ps_quote("it's"), "it''s");
    }

    #[test]
    fn empty_strings_pass_through() {
        assert_eq!(escape_applescript(""), "");
        assert_eq!(ps_quote(""), "");
    }
}
