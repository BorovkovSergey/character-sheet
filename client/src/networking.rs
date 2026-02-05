use bevy::ecs::message::{MessageReader, MessageWriter, Messages};
use bevy::log::{error, info, warn};
use bevy::prelude::*;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use shared::{deserialize, serialize, ClientMessage, ServerMessage};

use crate::state::{AppState, ConnectionStatus};

/// Resource holding WebSocket connection (non-send for WASM compatibility)
pub struct WebSocketConnection {
    pub sender: WsSender,
    pub receiver: WsReceiver,
}

/// Message to send to the server
#[derive(Message)]
pub struct SendMessage(pub ClientMessage);

/// Message when server message is received
#[derive(Message)]
pub struct ReceivedMessage(pub ServerMessage);

/// Message to trigger reconnection attempt
#[derive(Message)]
pub struct ReconnectRequest;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SendMessage>()
            .add_message::<ReceivedMessage>()
            .add_message::<ReconnectRequest>()
            .add_systems(Startup, connect_to_server)
            .add_systems(
                Update,
                (
                    receive_messages,
                    handle_server_messages.after(receive_messages),
                    send_messages,
                    handle_reconnect,
                ),
            );
    }
}

fn connect_to_server(world: &mut World) {
    // Update connection status
    {
        let mut state = world.resource_mut::<AppState>();
        state.connection_status = ConnectionStatus::Connecting;
    }

    // Get WebSocket URL based on current page location
    let ws_url = get_ws_url();

    info!("Connecting to WebSocket: {}", ws_url);

    match ewebsock::connect(&ws_url, ewebsock::Options::default()) {
        Ok((sender, receiver)) => {
            world.insert_non_send_resource(WebSocketConnection { sender, receiver });
            let mut state = world.resource_mut::<AppState>();
            state.connection_status = ConnectionStatus::Connected;
            info!("WebSocket connected");
        }
        Err(e) => {
            let mut state = world.resource_mut::<AppState>();
            state.connection_status = ConnectionStatus::Error(e.clone());
            error!("Failed to connect: {}", e);
        }
    }
}

/// Default fallback WebSocket URL when browser APIs fail
const DEFAULT_WS_URL: &str = "ws://localhost:8080/ws";

fn get_ws_url() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        // In browser, connect to same host
        // If browser APIs fail, fall back to default URL
        let Some(window) = web_sys::window() else {
            warn!("Failed to get window object, using fallback URL");
            return DEFAULT_WS_URL.to_string();
        };

        let location = window.location();

        let host = match location.host() {
            Ok(h) => h,
            Err(_) => {
                warn!("Failed to get host from location, using fallback URL");
                return DEFAULT_WS_URL.to_string();
            }
        };

        // In development, trunk serves on 8081 but server is on 8080
        // Detect this and connect to server directly
        if host.contains(":8081") {
            let server_host = host.replace(":8081", ":8080");
            info!(
                "Development mode detected, connecting to server at {}",
                server_host
            );
            return format!("ws://{}/ws", server_host);
        }

        let protocol = match location.protocol() {
            Ok(p) if p == "https:" => "wss",
            Ok(_) => "ws",
            Err(_) => {
                warn!("Failed to get protocol from location, defaulting to ws");
                "ws"
            }
        };

        format!("{}://{}/ws", protocol, host)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        DEFAULT_WS_URL.to_string()
    }
}

fn receive_messages(
    connection: Option<NonSend<WebSocketConnection>>,
    mut received_messages: MessageWriter<ReceivedMessage>,
    mut state: ResMut<AppState>,
) {
    let Some(conn) = connection else {
        return;
    };

    while let Some(event) = conn.receiver.try_recv() {
        match event {
            WsEvent::Message(WsMessage::Binary(data)) => {
                match deserialize::<ServerMessage>(&data) {
                    Ok(msg) => {
                        received_messages.write(ReceivedMessage(msg));
                    }
                    Err(e) => {
                        warn!("Failed to deserialize message: {}", e);
                    }
                }
            }
            WsEvent::Opened => {
                state.connection_status = ConnectionStatus::Connected;
                info!("WebSocket opened");
            }
            WsEvent::Closed => {
                state.connection_status = ConnectionStatus::Disconnected;
                warn!("WebSocket closed");
            }
            WsEvent::Error(e) => {
                state.connection_status = ConnectionStatus::Error(e.clone());
                error!("WebSocket error: {}", e);
            }
            _ => {}
        }
    }
}

fn send_messages(
    mut connection: Option<NonSendMut<WebSocketConnection>>,
    mut send_reader: MessageReader<SendMessage>,
) {
    let Some(ref mut conn) = connection else {
        return;
    };

    for SendMessage(msg) in send_reader.read() {
        match serialize(msg) {
            Ok(bytes) => {
                conn.sender.send(WsMessage::Binary(bytes));
            }
            Err(e) => {
                error!("Failed to serialize message: {}", e);
            }
        }
    }
}

fn handle_server_messages(
    mut received_reader: MessageReader<ReceivedMessage>,
    mut state: ResMut<AppState>,
) {
    for ReceivedMessage(msg) in received_reader.read() {
        match msg {
            ServerMessage::CharacterList { characters } => {
                info!("Received {} characters", characters.len());
                state.set_characters(characters.clone());
            }
            ServerMessage::CharacterCreated { character } => {
                info!("Character created: {}", character.name);
                state.add_character(character.clone());
            }
            ServerMessage::CharacterDeleted { id } => {
                info!("Character deleted: {}", id);
                state.remove_character(*id);
            }
            ServerMessage::Error { message } => {
                error!("Server error: {}", message);
            }
        }
    }
}

fn handle_reconnect(world: &mut World) {
    // Check if there's a reconnect request
    let should_reconnect = {
        let mut messages = world.resource_mut::<Messages<ReconnectRequest>>();
        let has_message = !messages.is_empty();
        if has_message {
            messages.clear();
        }
        has_message
    };

    if !should_reconnect {
        return;
    }

    // Remove existing connection if any
    world.remove_non_send_resource::<WebSocketConnection>();

    // Update connection status
    {
        let mut state = world.resource_mut::<AppState>();
        state.connection_status = ConnectionStatus::Connecting;
    }

    let ws_url = get_ws_url();
    info!("Reconnecting to WebSocket: {}", ws_url);

    match ewebsock::connect(&ws_url, ewebsock::Options::default()) {
        Ok((sender, receiver)) => {
            world.insert_non_send_resource(WebSocketConnection { sender, receiver });
            let mut state = world.resource_mut::<AppState>();
            state.connection_status = ConnectionStatus::Connected;
            info!("WebSocket reconnected");
        }
        Err(e) => {
            let mut state = world.resource_mut::<AppState>();
            state.connection_status = ConnectionStatus::Error(e.clone());
            error!("Failed to reconnect: {}", e);
        }
    }
}
