# axosm

4x browser game

## Usage / Testing

From https://chatgpt.com/c/692ad77a-dac4-832d-a152-77d026642e32

### Start Rust server:

cd back
cargo run --release

This creates game.db (SQLite), applies migrations, and seeds two players with IDs 1 and 2.

### In browser:

Invoke-WebRequest : The remote server returned an error: (404) Not Found.
At line:1 char:1
+ Invoke-WebRequest -Uri http://localhost:3000/api/state/1
+ ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : InvalidOperation: (System.Net.HttpWebRequest:HttpWebRequest) [Invoke-WebRequest], WebExc
   eption
    + FullyQualifiedErrorId : WebCmdletWebResponseException,Microsoft.PowerShell.Commands.InvokeWebRequestCommand

### PowerShell commands

Invoke-WebRequest -Uri http://localhost:3000/api/state/1

curl http://localhost:3000/api/state/1

TODO
- continue tauri tuto : https://v2.tauri.app/start/create-project/
- do vue tuto
- add vue to the project
- add tauri to the project : https://v2.tauri.app/start/create-project/#manual-setup-tauri-cli


## Project architecture

src/
├── main.rs
├── config.rs                  # App config, env vars
├── errors.rs                  # Global error type, impl IntoResponse
├── state.rs                   # AppState, DB pool
│
├── auth/
│   ├── mod.rs
│   ├── middleware.rs          # AuthPlayer extractor
│   ├── handlers.rs            # login, register, logout
│   └── service.rs             # hash password, verify token
│
├── db/                        # Raw DB models (FromRow)
│   ├── mod.rs
│   ├── player.rs
│   ├── planet.rs
│   ├── unit.rs
│   ├── alliance.rs
│   ├── resources.rs
│   └── session.rs
│
├── dto/                       # API shapes (Serialize/Deserialize)
│   ├── mod.rs
│   ├── player.rs              # PublicPlayerInfo, PlayerResponse
│   ├── planet.rs              # PlanetResponse, PlanetSummary
│   ├── alliance.rs            # AllianceResponse, CreateAllianceRequest
│   ├── resources.rs           # ResourceState, ProductionRates
│   └── game.rs                # GameState, InitResponse
│
├── handlers/                  # Axum route handlers (thin layer)
│   ├── mod.rs
│   ├── game.rs
│   ├── planet.rs
│   ├── fleet.rs
│   ├── alliance.rs
│   └── admin.rs
│
├── services/                  # Business logic
│   ├── mod.rs
│   ├── game.rs                # init_new_player, load_game_state
│   ├── resources.rs           # compute_resources, tick production
│   ├── battle.rs              # battle resolution
│   ├── fleet.rs               # movement, arrival
│   └── alliance.rs            # invite, kick, rank
│
├── game/                      # Pure game logic, no DB/HTTP
│   ├── mod.rs
│   ├── fog.rs                 # reveal_fog, visibility calc
│   ├── combat.rs              # damage formulas
│   ├── production.rs          # rate calculations
│   └── map.rs                 # planet tile generation
│
└── routes.rs                  # Router assembly, all .route() calls