use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use shared::{deserialize, serialize, ClientMessage, ServerMessage};
use tracing::{error, info, warn};

use crate::storage::CharacterStore;

pub async fn ws_handler(ws: WebSocketUpgrade, State(store): State<CharacterStore>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, store))
}

async fn handle_socket(socket: WebSocket, store: CharacterStore) {
    let (mut sender, mut receiver) = socket.split();

    info!("New WebSocket connection");

    // Send initial character list
    let characters = store.get_all().await;
    let msg = ServerMessage::CharacterList { characters };
    if let Ok(bytes) = serialize(&msg) {
        if sender.send(Message::Binary(bytes)).await.is_err() {
            return;
        }
    }

    // Handle incoming messages
    while let Some(result) = receiver.next().await {
        match result {
            Ok(Message::Binary(data)) => {
                if let Ok(client_msg) = deserialize::<ClientMessage>(&data) {
                    if let Some(response) = handle_message(client_msg, &store).await {
                        if let Ok(bytes) = serialize(&response) {
                            if sender.send(Message::Binary(bytes)).await.is_err() {
                                break;
                            }
                        }
                    }
                } else {
                    warn!("Failed to deserialize client message");
                }
            }
            Ok(Message::Close(_)) => {
                info!("Client disconnected");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    info!("WebSocket connection closed");
}

async fn handle_message(msg: ClientMessage, store: &CharacterStore) -> Option<ServerMessage> {
    match msg {
        ClientMessage::RequestCharacterList => {
            let characters = store.get_all().await;
            Some(ServerMessage::CharacterList { characters })
        }
        ClientMessage::CreateCharacter { name } => {
            if name.trim().is_empty() {
                return Some(ServerMessage::Error {
                    message: "Character name cannot be empty".to_string(),
                });
            }
            if name.len() > 100 {
                return Some(ServerMessage::Error {
                    message: "Character name cannot exceed 100 characters".to_string(),
                });
            }
            let character = store.create(name).await;
            Some(ServerMessage::CharacterCreated { character })
        }
        ClientMessage::DeleteCharacter { id } => {
            if store.delete(id).await {
                Some(ServerMessage::CharacterDeleted { id })
            } else {
                Some(ServerMessage::Error {
                    message: format!("Character with id {} not found", id),
                })
            }
        }
        ClientMessage::UpdateCharacter { character } => {
            if let Some(updated) = store.update(character).await {
                Some(ServerMessage::CharacterUpdated { character: updated })
            } else {
                Some(ServerMessage::Error {
                    message: "Character not found".to_string(),
                })
            }
        }
        ClientMessage::CreateWeapon { weapon } => {
            if let Err(e) = store.save_weapon(weapon).await {
                error!("Failed to save weapon: {e}");
            }
            None
        }
        ClientMessage::CreateEquipment { equipment } => {
            if let Err(e) = store.save_equipment(equipment).await {
                error!("Failed to save equipment: {e}");
            }
            None
        }
        ClientMessage::CreateItem { item } => {
            if let Err(e) = store.save_item(item).await {
                error!("Failed to save item: {e}");
            }
            None
        }
    }
}
