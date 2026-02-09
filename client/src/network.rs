use bevy::prelude::*;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use shared::{deserialize, ServerMessage, TraitRegistry};

use crate::character_select::CharacterList;

/// Holds the WebSocket sender/receiver pair as a non-send Bevy resource.
///
/// Stored as non-send because [`WsReceiver`] uses `std::sync::mpsc::Receiver`
/// internally, which is not `Sync`.
pub struct WsConnection {
    #[allow(dead_code)]
    pub sender: WsSender,
    pub receiver: WsReceiver,
}

#[derive(Resource)]
pub struct ClientTraitRegistry(pub TraitRegistry);

/// Buffer for server messages drained from the WebSocket.
/// Filled by the exclusive `drain_ws` system, consumed by `process_server_messages`.
#[derive(Resource, Default)]
struct PendingServerMessages(Vec<ServerMessage>);

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        let registry = TraitRegistry::load_from_str(include_str!("../../data/traits.json"))
            .expect("failed to parse embedded traits.json");
        app.insert_resource(ClientTraitRegistry(registry))
            .init_resource::<PendingServerMessages>()
            .add_systems(Startup, connect_to_server)
            .add_systems(Update, (drain_ws, process_server_messages).chain());
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

/// Minimal exclusive system: drains the WebSocket and buffers decoded messages.
fn drain_ws(world: &mut World) {
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
    let mut pending = world.resource_mut::<PendingServerMessages>();

    for event in events {
        match event {
            WsEvent::Opened => {
                info!("WebSocket connection opened");
            }
            WsEvent::Message(WsMessage::Binary(bytes)) => {
                match deserialize::<ServerMessage>(&bytes) {
                    Ok(msg) => pending.0.push(msg),
                    Err(e) => warn!("Failed to deserialize server message: {e}"),
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

    if should_remove {
        world.remove_non_send_resource::<WsConnection>();
        info!("Cleaned up WebSocket connection resource");
    }
}

/// Normal system: processes buffered server messages and updates game state.
fn process_server_messages(
    mut pending: ResMut<PendingServerMessages>,
    trait_registry: Res<ClientTraitRegistry>,
    mut character_list: ResMut<CharacterList>,
) {
    for msg in pending.0.drain(..) {
        match msg {
            ServerMessage::CharacterList { mut characters } => {
                info!("Received {} character(s) from server", characters.len());
                for c in &mut characters {
                    c.recalculate_effects(&trait_registry.0);
                }
                character_list.characters = characters;
            }
            ServerMessage::CharacterCreated { .. } => {
                warn!("Received unhandled CharacterCreated message");
            }
            ServerMessage::CharacterUpdated { .. } => {
                warn!("Received unhandled CharacterUpdated message");
            }
            ServerMessage::CharacterDeleted { .. } => {
                warn!("Received unhandled CharacterDeleted message");
            }
            ServerMessage::Error { message } => {
                error!("Server error: {message}");
            }
        }
    }
}
