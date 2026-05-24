use std::env;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionKind {
    X11,
    Wayland,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaylandCompositor {
    Gnome,
    Kde,
    Sway,
    Hyprland,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionInfo {
    pub kind: SessionKind,
    pub raw_session_type: String,
    pub compositor: WaylandCompositor,
    pub desktop_label: String,
}

pub fn detect() -> SessionInfo {
    detect_from_env(|name| env::var(name).ok())
}

pub fn detect_from_env<F>(get: F) -> SessionInfo
where
    F: Fn(&str) -> Option<String>,
{
    let raw_session_type = get("XDG_SESSION_TYPE").unwrap_or_else(|| "unknown".to_string());
    let session_type = raw_session_type.to_ascii_lowercase();
    let desktop = get("XDG_CURRENT_DESKTOP")
        .or_else(|| get("DESKTOP_SESSION"))
        .unwrap_or_else(|| "unknown".to_string());
    let desktop_lower = desktop.to_ascii_lowercase();

    let kind = match session_type.as_str() {
        "x11" => SessionKind::X11,
        "wayland" => SessionKind::Wayland,
        _ => SessionKind::Unknown,
    };

    let compositor = if matches!(kind, SessionKind::Wayland) {
        detect_wayland_compositor(&get, &desktop_lower)
    } else {
        WaylandCompositor::Unknown
    };

    SessionInfo {
        kind,
        raw_session_type,
        compositor,
        desktop_label: desktop,
    }
}

fn detect_wayland_compositor<F>(get: &F, desktop_lower: &str) -> WaylandCompositor
where
    F: Fn(&str) -> Option<String>,
{
    if get("HYPRLAND_INSTANCE_SIGNATURE").is_some() || desktop_lower.contains("hypr") {
        return WaylandCompositor::Hyprland;
    }
    if get("SWAYSOCK").is_some() || desktop_lower.contains("sway") {
        return WaylandCompositor::Sway;
    }
    if get("KDE_FULL_SESSION").is_some() || desktop_lower.contains("kde") || desktop_lower.contains("plasma") {
        return WaylandCompositor::Kde;
    }
    if get("GNOME_DESKTOP_SESSION_ID").is_some() || desktop_lower.contains("gnome") {
        return WaylandCompositor::Gnome;
    }

    WaylandCompositor::Unknown
}

pub fn compositor_label(compositor: &WaylandCompositor) -> &'static str {
    match compositor {
        WaylandCompositor::Gnome => "gnome",
        WaylandCompositor::Kde => "kde",
        WaylandCompositor::Sway => "sway",
        WaylandCompositor::Hyprland => "hyprland",
        WaylandCompositor::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_hyprland_on_wayland() {
        let info = detect_from_env(|name| match name {
            "XDG_SESSION_TYPE" => Some("wayland".to_string()),
            "HYPRLAND_INSTANCE_SIGNATURE" => Some("abc".to_string()),
            "XDG_CURRENT_DESKTOP" => Some("Hyprland".to_string()),
            _ => None,
        });

        assert!(matches!(info.kind, SessionKind::Wayland));
        assert!(matches!(info.compositor, WaylandCompositor::Hyprland));
    }

    #[test]
    fn detects_sway_on_wayland() {
        let info = detect_from_env(|name| match name {
            "XDG_SESSION_TYPE" => Some("wayland".to_string()),
            "SWAYSOCK" => Some("/tmp/sway.sock".to_string()),
            _ => None,
        });

        assert!(matches!(info.kind, SessionKind::Wayland));
        assert!(matches!(info.compositor, WaylandCompositor::Sway));
    }

    #[test]
    fn detects_x11_without_wayland_compositor() {
        let info = detect_from_env(|name| match name {
            "XDG_SESSION_TYPE" => Some("x11".to_string()),
            "XDG_CURRENT_DESKTOP" => Some("GNOME".to_string()),
            _ => None,
        });

        assert!(matches!(info.kind, SessionKind::X11));
        assert!(matches!(info.compositor, WaylandCompositor::Unknown));
    }
}
