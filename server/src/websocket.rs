use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use shared::{deserialize, serialize, ClientMessage, ServerMessage};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::storage::CharacterStore;
use crate::AppState;

/// Maximum portrait size in bytes (512KB).
const MAX_PORTRAIT_SIZE: usize = 512 * 1024;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state.store, state.admin_password))
}

async fn handle_socket(socket: WebSocket, store: CharacterStore, admin_password: Option<Arc<str>>) {
    let (mut sender, mut receiver) = socket.split();

    // If no admin password is configured, start authenticated (backwards compatible).
    let mut authenticated = admin_password.is_none();

    info!(
        "New WebSocket connection (authenticated: {})",
        authenticated
    );

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
                    let responses =
                        handle_message(client_msg, &store, &admin_password, &mut authenticated)
                            .await;
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

/// Returns true if the message is a mutation (write operation).
fn is_mutation(msg: &ClientMessage) -> bool {
    matches!(
        msg,
        ClientMessage::CreateCharacter { .. }
            | ClientMessage::UpdateCharacter { .. }
            | ClientMessage::DeleteCharacter { .. }
            | ClientMessage::DeleteVersion { .. }
            | ClientMessage::CreateWeapon { .. }
            | ClientMessage::CreateEquipment { .. }
            | ClientMessage::CreateItem { .. }
            | ClientMessage::UploadPortrait { .. }
    )
}

async fn handle_message(
    msg: ClientMessage,
    store: &CharacterStore,
    admin_password: &Option<Arc<str>>,
    authenticated: &mut bool,
) -> Vec<ServerMessage> {
    // Handle authentication
    if let ClientMessage::Authenticate { password } = &msg {
        if let Some(expected) = admin_password {
            let success = password == expected.as_ref();
            *authenticated = success;
            if success {
                info!("Client authenticated successfully");
            } else {
                warn!("Client authentication failed");
            }
            return vec![ServerMessage::AuthResult { success }];
        } else {
            // No password configured, always succeed
            *authenticated = true;
            return vec![ServerMessage::AuthResult { success: true }];
        }
    }

    // Reject mutations from unauthenticated connections
    if !*authenticated && is_mutation(&msg) {
        return vec![ServerMessage::Error {
            message: "Not authenticated — enable write access first".to_string(),
        }];
    }

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
            let trimmed = name.trim().to_string();
            if store.character_name_exists(&trimmed).await {
                return vec![ServerMessage::Error {
                    message: format!("Character with name \"{}\" already exists", trimmed),
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
        ClientMessage::CreateWeapon { weapon } => match store.save_weapon(weapon).await {
            Err(e) => vec![ServerMessage::Error { message: e }],
            Ok(()) => vec![],
        },
        ClientMessage::CreateEquipment { equipment } => {
            match store.save_equipment(equipment).await {
                Err(e) => vec![ServerMessage::Error { message: e }],
                Ok(()) => vec![],
            }
        }
        ClientMessage::CreateItem { item } => match store.save_item(item).await {
            Err(e) => vec![ServerMessage::Error { message: e }],
            Ok(()) => vec![],
        },
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
        ClientMessage::Authenticate { .. } => {
            // Already handled above, but needed for exhaustive match
            vec![]
        }
    }
}
