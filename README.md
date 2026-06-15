# Iroh Retro Board

A serverless, collaborative agile retrospective board that runs entirely in the browser. Real-time synchronization between participants is handled completely decentralized via peer-to-peer (P2P) connections using the Iroh network protocol.

There is no registration required, no central database, and zero server infrastructure costs.

---

## Features

* **100% Serverless:** Functions entirely as a client-side WebAssembly (WASM) application that can be hosted free of charge on GitHub Pages.
* **Real-Time P2P Synchronization:** Direct discovery and data exchange between peers via Iroh, eliminating the need for a central relational or document server.
* **Persistent Usernames:** Manually updated usernames are saved to the browser local storage to persist across page reloads.
* **Responsive Layout:** A clean, dark-mode user interface based on Flexbox with mathematically aligned input controls.
* **Board Mechanics:**
    * Add cards into two distinct columns: "What went well" and "What could be improved".
    * Upvote system for cards with integrated protection against duplicate votes from the same peer.
    * Real-time participant list displayed in the sidebar.

---

## Tech Stack

* **Frontend Framework:** Leptos (Rust WebAssembly Framework)
* **P2P Networking:** Iroh (For direct end-to-end connections)
* **Build Tool:** Trunk (WASM bundler for Rust)
* **Language:** Rust

---

## Local Development

### Prerequisites

Ensure that Rust and the WebAssembly target infrastructure are installed on your system:

```bash
# Add the WASM target
rustup target add wasm32-unknown-unknown

# Install Trunk if not already present
cargo install --locked trunk

trunk serve --public-url / --open
```
