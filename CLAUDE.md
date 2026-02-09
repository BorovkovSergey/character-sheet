## Development Rules

- All documentation must be in English
- Keep files small and well-organized
- Prefer module directories (`foo/mod.rs` + submodules) over monolithic files for complex domains
- Use `BTreeMap` instead of `HashMap` for deterministic ordering

## Bevy ECS Rules

### Components over God Resources
- Never store complex game state inside a single `Resource`. Each logical piece of data (HP, mana, level, stats, etc.) must be a separate `Component` on an Entity
- Game objects (characters, items, enemies) must be Entities with Components, not plain structs inside a `Vec` or `Resource`
- If a struct has more than 3-4 fields that change independently, split it into multiple Components

### Use Bevy States for screen/mode management
- Use `#[derive(States)]` for app screens, game phases, and modal states
- Use `run_if(in_state(...))` instead of manual `if !matches!(...)` guards in systems
- Use `OnEnter(State)` / `OnExit(State)` for setup/teardown logic
- Transition with `ResMut<NextState<T>>`, never by mutating a Resource enum directly

### Prefer normal systems over exclusive systems
- Only use exclusive systems (`fn(world: &mut World)`) when absolutely required (e.g., inserting non-send resources)
- Separate I/O (WebSocket drain, file reads) from game logic — I/O in exclusive system, logic in normal system
- Use buffer `Resource` or `Events` to pass data from exclusive systems to normal ones

### Leverage change detection
- Use `Changed<T>` and `Added<T>` query filters for reactive systems instead of recalculating every frame
- Keep components granular to make change detection useful — `Changed<Hp>` is actionable, `Changed<GodStruct>` is not

### No `#[allow(dead_code)]` to suppress real issues
- If a field is stored but never read, fix the code to actually use it or remove the field
- `#[allow(dead_code)]` is only acceptable for components that are stored in ECS for future query use but not yet queried

### Shared crate boundary
- The `shared` crate must remain Bevy-free (used by both client and server)
- Client must have a mapping layer: convert `shared` types into ECS Components at the network boundary (on message receive / entity spawn)
- Never use `shared` structs directly as Bevy Resources for mutable game state