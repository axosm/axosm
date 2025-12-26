# axosm

4x browser game

## Usage / Testing

From https://chatgpt.com/c/692ad77a-dac4-832d-a152-77d026642e32

### Start Rust server:

cd back
cargo run --release

This creates game.db (SQLite), applies migrations, and seeds two players with IDs 1 and 2.

### Start frontend:

cd front
npx vite

Open http://127.0.0.1:8000 in your browser.

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
