use bevy::prelude::*;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use shared::{deserialize, ServerMessage};

use crate::character_select::CharacterList;

/// Holds the WebSocket sender/receiver pair as a non-send Bevy resource.
///
/// Stored as non-send because [`WsReceiver`] uses `std::sync::mpsc::Receiver`
/// internally, which is not `Sync`.
pub struct WsConnection {
    /// Sender for transmitting messages to the server.
    /// Will be used when client-to-server messaging is implemented.
    #[allow(dead_code)]
    pub sender: WsSender,
    pub receiver: WsReceiver,
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, connect_to_server)
            .add_systems(Update, receive_messages);
    }
}

fn server_url() -> Result<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window().ok_or("no browser window available")?;
        let location = window.location();
        let protocol = location
            .protocol()
            .map_err(|_| "failed to read location protocol")?;
        let host = location
            .host()
            .map_err(|_| "failed to read location host")?;
        let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
        Ok(format!("{ws_protocol}//{host}/ws"))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Ok("ws://127.0.0.1:8080/ws".to_string())
    }
}

fn connect_to_server(world: &mut World) {
    let url = match server_url() {
        Ok(url) => url,
        Err(err) => {
            error!("Failed to determine server URL: {err}");
            return;
        }
    };
    info!("Connecting to server at {url}");

    match ewebsock::connect(&url, ewebsock::Options::default()) {
        Ok((sender, receiver)) => {
            info!("WebSocket connection initiated");
            world.insert_non_send_resource(WsConnection { sender, receiver });
        }
        Err(err) => {
            error!("Failed to connect to server: {err}");
        }
    }
}

fn receive_messages(world: &mut World) {
    let events = {
        let Some(conn) = world.get_non_send_resource::<WsConnection>() else {
            return;
        };
        let mut events = Vec::new();
        while let Some(event) = conn.receiver.try_recv() {
            events.push(event);
        }
        events
    };

    let mut should_remove = false;
    let mut new_characters = None;

    for event in events {
        match event {
            WsEvent::Opened => {
                info!("WebSocket connection opened");
            }
            WsEvent::Message(WsMessage::Binary(bytes)) => {
                match deserialize::<ServerMessage>(&bytes) {
                    Ok(ServerMessage::CharacterList { characters }) => {
                        info!("Received {} character(s) from server", characters.len());
                        new_characters = Some(characters);
                    }
                    Ok(ServerMessage::CharacterCreated { .. }) => {
                        warn!("Received unhandled CharacterCreated message");
                    }
                    Ok(ServerMessage::CharacterUpdated { .. }) => {
                        warn!("Received unhandled CharacterUpdated message");
                    }
                    Ok(ServerMessage::CharacterDeleted { .. }) => {
                        warn!("Received unhandled CharacterDeleted message");
                    }
                    Ok(ServerMessage::Error { message }) => {
                        error!("Server error: {message}");
                    }
                    Err(e) => {
                        warn!("Failed to deserialize server message: {e}");
                    }
                }
            }
            WsEvent::Error(err) => {
                error!("WebSocket error: {err}");
                should_remove = true;
            }
            WsEvent::Closed => {
                info!("WebSocket connection closed");
                should_remove = true;
            }
            _ => {}
        }
    }

    if let Some(mut characters) = new_characters {
        for c in &mut characters {
            c.recalculate_effects();
        }
        world.resource_mut::<CharacterList>().characters = characters;
    }

    if should_remove {
        world.remove_non_send_resource::<WsConnection>();
        info!("Cleaned up WebSocket connection resource");
    }
}
