# axosm

4x browser game

## Usage / Testing

### Start Rust server:

From https://chatgpt.com/c/692ad77a-dac4-832d-a152-77d026642e32

cd rust-server
cargo run --release


This creates game.db (SQLite), applies migrations, and seeds two players with IDs 1 and 2.

### Start frontend:

cd frontend
deno run --allow-net --allow-read --unstable https://deno.land/std@0.205.0/http/file_server.ts


Open http://127.0.0.1:8000 in your browser.

### In browser:

Use Player ID 1 for first window — click Connect SSE.

Open a second window/tab to the same page and set Player ID to 2 and click Connect SSE.

Click a tile to move a unit; movement arrival is 10 seconds after you click (server schedules arrival_time = now + 10s).

When both units end in same tile (after arrival), both connected clients will receive an SSE encounter event.