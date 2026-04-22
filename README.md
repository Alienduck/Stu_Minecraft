Stu_Minecraft

A Minecraft clone developed in Rust using the Bevy game engine. This project features a full multiplayer architecture with an authoritative server and procedural world generation.
How to Play

The project is structured as a workspace with separate server and client executables. You must run both to play.
Prerequisites

Ensure you have the Rust toolchain and Cargo installed.
1. Launch the Server

Open a terminal at the project root and start the headless server. It manages the world state and player connections:

```powershell
cargo run --release -p server
```

The server listens on UDP port 25565 by default.
2. Launch the Client(s)

Open a second terminal to launch the graphical client. You must provide a username as a command-line argument:

```powershell
cargo run --release -p client -- PlayerName
```

To test multiplayer locally, you can open additional terminals and launch more clients with different names.

Note: The initial compilation may take several minutes, but subsequent builds will be significantly faster.
Architecture

The project is organized into three main modules within a Cargo workspace:

1. shared

    - Contains logic used by both the client and server.

    - Procedural Generation: Uses Perlin noise to generate terrain deterministically based on a seed. This allows clients to generate the baseline terrain locally instead of downloading entire chunks.

    - Network Protocol: Defines the packets used for communication, which are serialized using serde and bincode.

2. server

    - A headless UDP server that manages authoritative world state.

    - It validates player actions (such as breaking or placing blocks), maintains a list of block modifications (deltas), and broadcasts player movements to all connected clients.

3. client

    - Uses the Bevy engine for 3D rendering and input handling.

    - Networking: Network communication runs on a dedicated background thread to prevent blocking the main rendering loop. It communicates with Bevy systems through asynchronous channels.

    - Rendering: Generates local chunk meshes and applies server-side block updates in real-time.

How to Contribute

Contributions are welcome, whether for adding new block types, improving terrain generation, or optimizing mesh construction.
Guidelines

    A basic understanding of the Rust programming language is required.

    Familiarity with the Entity-Component-System (ECS) architecture and the Bevy engine is recommended.

Steps to Contribute

    Fork the repository.

    Clone your fork locally: git clone https://github.com/YOUR_USERNAME/Stu_Minecraft.git.

    Create a new branch for your feature: git checkout -b feature-name.

    Commit your changes and ensure they work in a multiplayer environment.

    Push to your branch: git push origin feature-name.

    Open a Pull Request for review.

If you encounter bugs or have suggestions, please open an [issue](https://github.com/Alienduck/Stu_Minecraft/issues).