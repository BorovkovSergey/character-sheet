use bevy::prelude::*;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use shared::character::SkillRegistry;
use shared::{
    deserialize, AbilityRegistry, ClientMessage, EquipmentRegistry, ItemRegistry, ServerMessage,
    TraitRegistry, WeaponRegistry,
};

use crate::character_select::CharacterList;

/// Holds the WebSocket sender/receiver pair as a non-send Bevy resource.
///
/// Stored as non-send because [`WsReceiver`] uses `std::sync::mpsc::Receiver`
/// internally, which is not `Sync`.
pub struct WsConnection {
    pub sender: WsSender,
    pub receiver: WsReceiver,
}

#[derive(Resource)]
pub struct ClientTraitRegistry(pub TraitRegistry);

#[derive(Resource)]
pub struct ClientSkillRegistry(pub SkillRegistry);

#[derive(Resource)]
pub struct ClientAbilityRegistry(pub AbilityRegistry);

#[derive(Resource)]
pub struct ClientWeaponRegistry(pub WeaponRegistry);

#[derive(Resource)]
pub struct ClientEquipmentRegistry(pub EquipmentRegistry);

#[derive(Resource)]
#[allow(dead_code)]
pub struct ClientItemRegistry(pub ItemRegistry);

/// Buffer for server messages drained from the WebSocket.
/// Filled by `drain_ws`, consumed by `process_server_messages`.
#[derive(Resource, Default)]
struct PendingServerMessages(Vec<ServerMessage>);

/// Buffer for client messages to send via WebSocket.
/// Filled by game systems, drained by `send_client_messages`.
#[derive(Resource, Default)]
pub struct PendingClientMessages(pub Vec<ClientMessage>);

#[derive(Resource)]
struct ReconnectTimer(Timer);

impl Default for ReconnectTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(3.0, TimerMode::Repeating))
    }
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        let trait_reg = TraitRegistry::load_from_str(include_str!("../../data/traits.json"))
            .expect("failed to parse embedded traits.json");
        let skill_reg = SkillRegistry::load_from_str(include_str!("../../data/skills.json"))
            .expect("failed to parse embedded skills.json");
        let ability_reg = AbilityRegistry::load_from_str(include_str!("../../data/abilities.json"))
            .expect("failed to parse embedded abilities.json");
        let weapon_reg = WeaponRegistry::load_from_str(include_str!("../../data/weapons.json"))
            .expect("failed to parse embedded weapons.json");
        let equipment_reg =
            EquipmentRegistry::load_from_str(include_str!("../../data/equipment.json"))
                .expect("failed to parse embedded equipment.json");
        let item_reg = ItemRegistry::load_from_str(include_str!("../../data/items.json"))
            .expect("failed to parse embedded items.json");
        app.insert_resource(ClientTraitRegistry(trait_reg))
            .insert_resource(ClientSkillRegistry(skill_reg))
            .insert_resource(ClientAbilityRegistry(ability_reg))
            .insert_resource(ClientWeaponRegistry(weapon_reg))
            .insert_resource(ClientEquipmentRegistry(equipment_reg))
            .insert_resource(ClientItemRegistry(item_reg))
            .init_resource::<PendingServerMessages>()
            .init_resource::<PendingClientMessages>()
            .init_resource::<ReconnectTimer>()
            .add_systems(Startup, connect_to_server)
            .add_systems(
                Update,
                (drain_ws, process_server_messages, send_client_messages).chain(),
            )
            .add_systems(Update, attempt_reconnect);
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

/// Drains the WebSocket and buffers decoded messages.
fn drain_ws(
    conn: Option<NonSend<WsConnection>>,
    mut pending: ResMut<PendingServerMessages>,
    mut commands: Commands,
) {
    let Some(conn) = conn else { return };

    let mut should_remove = false;
    while let Some(event) = conn.receiver.try_recv() {
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
        commands.queue(|world: &mut World| {
            world.remove_non_send_resource::<WsConnection>();
        });
        info!("Cleaned up WebSocket connection resource");
    }
}

/// Normal system: processes buffered server messages and updates game state.
fn process_server_messages(
    mut pending: ResMut<PendingServerMessages>,
    mut character_list: ResMut<CharacterList>,
) {
    for msg in pending.0.drain(..) {
        match msg {
            ServerMessage::CharacterList { characters } => {
                info!("Received {} character(s) from server", characters.len());
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

/// Sends buffered client messages over the WebSocket.
fn send_client_messages(
    mut conn: Option<NonSendMut<WsConnection>>,
    mut pending: ResMut<PendingClientMessages>,
) {
    let Some(conn) = conn.as_mut() else { return };
    for msg in pending.0.drain(..) {
        if let Ok(bytes) = shared::serialize(&msg) {
            conn.sender.send(WsMessage::Binary(bytes));
        }
    }
}

/// Exclusive system: reconnects if the WebSocket is gone.
/// Must be exclusive because `WsSender`/`WsReceiver` are not `Send` on WASM
/// (`Rc<WebSocket>`), so `commands.queue()` cannot move them across threads.
fn attempt_reconnect(world: &mut World) {
    if world.get_non_send_resource::<WsConnection>().is_some() {
        world.resource_mut::<ReconnectTimer>().0.reset();
        return;
    }

    let delta = world.resource::<Time>().delta();
    world.resource_mut::<ReconnectTimer>().0.tick(delta);
    if !world.resource::<ReconnectTimer>().0.just_finished() {
        return;
    }

    let url = match server_url() {
        Ok(url) => url,
        Err(err) => {
            error!("Failed to determine server URL: {err}");
            return;
        }
    };
    info!("Attempting to reconnect to {url}");

    match ewebsock::connect(&url, ewebsock::Options::default()) {
        Ok((sender, receiver)) => {
            info!("Reconnected to server");
            world.insert_non_send_resource(WsConnection { sender, receiver });
        }
        Err(err) => {
            warn!("Reconnection failed: {err}");
        }
    }
}
