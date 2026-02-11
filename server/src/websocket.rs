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

    // Send character summaries on connect
    let summaries = store.get_all_summaries().await;
    let msg = ServerMessage::CharacterList {
        characters: summaries,
    };
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
            let summaries = store.get_all_summaries().await;
            Some(ServerMessage::CharacterList {
                characters: summaries,
            })
        }
        ClientMessage::RequestVersionList { id } => match store.get_version_list(id).await {
            Some(versions) => Some(ServerMessage::VersionList { id, versions }),
            None => Some(ServerMessage::Error {
                message: format!("Character {} not found", id),
            }),
        },
        ClientMessage::RequestCharacterVersion { id, version } => {
            match store.get_character_version(id, version).await {
                Some(cv) => Some(ServerMessage::CharacterVersion {
                    id,
                    version: cv.version,
                    saved_at: cv.saved_at,
                    character: Box::new(cv.character),
                }),
                None => Some(ServerMessage::Error {
                    message: "Version not found".to_string(),
                }),
            }
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
            let summary = store.create(name).await;
            Some(ServerMessage::CharacterCreated { summary })
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
        ClientMessage::DeleteVersion { id, version } => {
            match store.delete_version(id, version).await {
                Some(true) => Some(ServerMessage::VersionDeleted { id, version }),
                Some(false) => Some(ServerMessage::Error {
                    message: format!("Version {} not found", version),
                }),
                None => Some(ServerMessage::Error {
                    message: "Character not found".to_string(),
                }),
            }
        }
        ClientMessage::UpdateCharacter { character } => match store.update(character).await {
            Some(summary) => Some(ServerMessage::CharacterUpdated { summary }),
            None => Some(ServerMessage::Error {
                message: "Character not found".to_string(),
            }),
        },
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
