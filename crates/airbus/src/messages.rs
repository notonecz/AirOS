use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BusMessage {
    Notify(NotificationPayload),
    SetDockBadge { count: u32 },
    RegisterMenu { items: Vec<MenuItem> },
    MenuItemClicked { id: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notify_roundtrip_msgpack() {
        let msg = BusMessage::Notify(NotificationPayload {
            title: "Test".to_string(),
            body: "Hello".to_string(),
            icon: None,
        });
        let encoded = rmp_serde::to_vec(&msg).unwrap();
        let decoded: BusMessage = rmp_serde::from_slice(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn dock_badge_roundtrip_msgpack() {
        let msg = BusMessage::SetDockBadge { count: 5 };
        let encoded = rmp_serde::to_vec(&msg).unwrap();
        let decoded: BusMessage = rmp_serde::from_slice(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn register_menu_roundtrip_msgpack() {
        let msg = BusMessage::RegisterMenu {
            items: vec![
                MenuItem {
                    id: "open".to_string(),
                    label: "Open".to_string(),
                    enabled: true,
                },
                MenuItem {
                    id: "close".to_string(),
                    label: "Close".to_string(),
                    enabled: false,
                },
            ],
        };
        let encoded = rmp_serde::to_vec(&msg).unwrap();
        let decoded: BusMessage = rmp_serde::from_slice(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn notify_with_icon() {
        let msg = BusMessage::Notify(NotificationPayload {
            title: "Update".to_string(),
            body: "New version available".to_string(),
            icon: Some("/assets/icons/update.png".to_string()),
        });
        let encoded = rmp_serde::to_vec(&msg).unwrap();
        let decoded: BusMessage = rmp_serde::from_slice(&encoded).unwrap();
        if let BusMessage::Notify(p) = decoded {
            assert_eq!(p.icon, Some("/assets/icons/update.png".to_string()));
        } else {
            panic!("wrong variant");
        }
    }
}
