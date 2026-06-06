// crates/airshell/src/bus_handler.rs

use crate::topbar::ShellState;
use airbus::messages::{BusMessage, NotificationPayload};

/// Zpracuje příchozí BusMessage od aplikace a aktualizuje ShellState.
/// `app_id` identifikuje odesílatele (název/ID aplikace).
pub fn handle_bus_message(state: &mut ShellState, app_id: &str, msg: BusMessage) {
    match msg {
        BusMessage::Notify(NotificationPayload { title, body, icon }) => {
            state.notifications.push(title, body, icon);
        }
        BusMessage::SetDockBadge { count } => {
            state.dock.set_badge(app_id, count);
        }
        BusMessage::RegisterMenu { items } => {
            state.topbar.register_menu(app_id.to_string(), items);
        }
        BusMessage::MenuItemClicked { .. } => {
            // Odchozí zpráva (AirShell → app) — příchozím směrem ignorujeme.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use airbus::messages::{MenuItem, NotificationPayload};

    #[test]
    fn handle_notify_pushes_notification() {
        let mut state = ShellState::new();
        handle_bus_message(
            &mut state,
            "myapp",
            BusMessage::Notify(NotificationPayload {
                title: "Hello".into(),
                body: "World".into(),
                icon: None,
            }),
        );
        assert_eq!(state.notifications.len(), 1);
        let entry = state.notifications.entries().front().unwrap();
        assert_eq!(entry.title, "Hello");
        assert_eq!(entry.body, "World");
    }

    #[test]
    fn handle_set_dock_badge_updates_dock() {
        let mut state = ShellState::new();
        // App must be running in dock before badge can be set
        state.dock.set_running("myapp", "My App", true);
        handle_bus_message(&mut state, "myapp", BusMessage::SetDockBadge { count: 3 });
        assert_eq!(state.dock.get("myapp").unwrap().badge, Some(3));
    }

    #[test]
    fn handle_register_menu_updates_topbar() {
        let mut state = ShellState::new();
        handle_bus_message(
            &mut state,
            "finder",
            BusMessage::RegisterMenu {
                items: vec![MenuItem {
                    id: "open".into(),
                    label: "Open".into(),
                    enabled: true,
                }],
            },
        );
        state.topbar.set_active_app(Some("finder".into()));
        let menu = state.topbar.active_menu().unwrap();
        assert_eq!(menu.len(), 1);
        assert_eq!(menu[0].id, "open");
    }
}
