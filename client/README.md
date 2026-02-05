# Client

Bevy + egui client for D&D Character Sheet.

## Architecture Rules

### Data Persistence

**All character data changes are applied locally.** Server synchronization happens only on explicit user action ("Save" button).

This means:
- UI directly modifies `state.characters`
- `UpdateCharacter` network requests are sent only when "Save" is clicked
- User controls when data is sent to server

### Structure

- `main.rs` - entry point, Bevy app setup
- `state.rs` - `AppState` resource with application data
- `networking.rs` - WebSocket connection and message handling
- `ui.rs` - egui interface
