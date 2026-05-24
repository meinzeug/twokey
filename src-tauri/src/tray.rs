use tauri::{
    menu::{MenuBuilder, MenuEvent},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Emitter, Manager,
};

pub fn setup(app: &App) -> Result<(), String> {
    let menu = MenuBuilder::new(app)
        .text("show_overlay", "Overlay zeigen")
        .text("open_settings", "Einstellungen")
        .separator()
        .text("quit", "Beenden")
        .build()
        .map_err(|error| format!("Tray-Menue konnte nicht erstellt werden: {error}"))?;

    let icon = app
        .default_window_icon()
        .ok_or_else(|| "Standard-Icon fuer Tray fehlt".to_string())?
        .clone();

    TrayIconBuilder::with_id("twokey-tray")
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| handle_menu_event(app, event))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = show_overlay(tray.app_handle());
            }
        })
        .build(app)
        .map_err(|error| format!("Tray-Icon konnte nicht erstellt werden: {error}"))?;

    Ok(())
}

fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        "show_overlay" => {
            let _ = show_overlay(app);
        }
        "open_settings" => {
            if let Some(window) = app.get_webview_window("settings") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

fn show_overlay(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("overlay")
        .ok_or_else(|| "Overlay-Fenster ist nicht konfiguriert".to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    let _ = app.emit("twokey://tray", "overlay-shown");
    Ok(())
}
