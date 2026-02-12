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

/// Maximum portrait size in bytes (512KB).
const MAX_PORTRAIT_SIZE: usize = 512 * 1024;

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
                    let responses = handle_message(client_msg, &store).await;
                    for response in responses {
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

async fn handle_message(msg: ClientMessage, store: &CharacterStore) -> Vec<ServerMessage> {
    match msg {
        ClientMessage::RequestCharacterList => {
            let summaries = store.get_all_summaries().await;
            vec![ServerMessage::CharacterList {
                characters: summaries,
            }]
        }
        ClientMessage::RequestVersionList { id } => match store.get_version_list(id).await {
            Some(versions) => vec![ServerMessage::VersionList { id, versions }],
            None => vec![ServerMessage::Error {
                message: format!("Character {} not found", id),
            }],
        },
        ClientMessage::RequestCharacterVersion { id, version } => {
            match store.get_character_version(id, version).await {
                Some(cv) => {
                    let mut msgs = vec![ServerMessage::CharacterVersion {
                        id,
                        version: cv.version,
                        saved_at: cv.saved_at,
                        character: Box::new(cv.character),
                    }];
                    // Auto-include portrait if one exists
                    if let Some(png_data) = store.load_portrait(id).await {
                        msgs.push(ServerMessage::PortraitData { id, png_data });
                    }
                    msgs
                }
                None => vec![ServerMessage::Error {
                    message: "Version not found".to_string(),
                }],
            }
        }
        ClientMessage::CreateCharacter {
            name,
            race,
            class,
            stats,
            skills,
            traits,
        } => {
            if name.trim().is_empty() {
                return vec![ServerMessage::Error {
                    message: "Character name cannot be empty".to_string(),
                }];
            }
            if name.len() > 100 {
                return vec![ServerMessage::Error {
                    message: "Character name cannot exceed 100 characters".to_string(),
                }];
            }
            let summary = store.create(name, race, class, stats, skills, traits).await;
            vec![ServerMessage::CharacterCreated { summary }]
        }
        ClientMessage::DeleteCharacter { id } => {
            if store.delete(id).await {
                vec![ServerMessage::CharacterDeleted { id }]
            } else {
                vec![ServerMessage::Error {
                    message: format!("Character with id {} not found", id),
                }]
            }
        }
        ClientMessage::DeleteVersion { id, version } => {
            match store.delete_version(id, version).await {
                Some(true) => vec![ServerMessage::VersionDeleted { id, version }],
                Some(false) => vec![ServerMessage::Error {
                    message: format!("Version {} not found", version),
                }],
                None => vec![ServerMessage::Error {
                    message: "Character not found".to_string(),
                }],
            }
        }
        ClientMessage::UpdateCharacter { character } => match store.update(character).await {
            Some(summary) => vec![ServerMessage::CharacterUpdated { summary }],
            None => vec![ServerMessage::Error {
                message: "Character not found".to_string(),
            }],
        },
        ClientMessage::CreateWeapon { weapon } => {
            if let Err(e) = store.save_weapon(weapon).await {
                error!("Failed to save weapon: {e}");
            }
            vec![]
        }
        ClientMessage::CreateEquipment { equipment } => {
            if let Err(e) = store.save_equipment(equipment).await {
                error!("Failed to save equipment: {e}");
            }
            vec![]
        }
        ClientMessage::CreateItem { item } => {
            if let Err(e) = store.save_item(item).await {
                error!("Failed to save item: {e}");
            }
            vec![]
        }
        ClientMessage::UploadPortrait { id, png_data } => {
            if png_data.len() > MAX_PORTRAIT_SIZE {
                return vec![ServerMessage::Error {
                    message: "Portrait too large (max 512KB)".to_string(),
                }];
            }
            store.save_portrait(id, &png_data).await;
            vec![ServerMessage::PortraitData { id, png_data }]
        }
        ClientMessage::RequestPortrait { id } => match store.load_portrait(id).await {
            Some(png_data) => vec![ServerMessage::PortraitData { id, png_data }],
            None => vec![],
        },
    }
}
