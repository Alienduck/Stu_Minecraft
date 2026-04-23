# Stu_Minecraft

*A Minecraft clone developed in Rust using the Bevy game engine. This project features a full multiplayer architecture with an authoritative server and procedural world generation.*

## How to Play

### Locally
The project is structured as a workspace with separate server and client executables. You must run both to play.
Prerequisites

Ensure you have the Rust toolchain and Cargo installed.
1. Launch the Server

Open a terminal at the project root and start the headless server. It manages the world state and player connections:

```powershell
cargo run --release -p server
```

The server listens on UDP port 25565 by default.\

2. Launch the Client(s)

Open a second terminal to launch the graphical client. You must provide a username as a command-line argument:

```powershell
cargo run --release -p client -- PlayerName
```

To test multiplayer on same machine, you can open additional terminals and launch more clients with different names.

Note: The initial compilation may take several minutes, but subsequent builds will be significantly faster.

### Server

*Yes you can actually play with anyone everywhere*

To do that you'll need [playit](https://playit.gg/download/windows) you need to download it and install.

(if you have a message with window saying it's dangerous, install anyway, I promise it's not)

Then you can run Playit and it will give you a link that you need to put in your browser and then follow the instructions.

For the options that you need to put:
- `Name`: [NAME_OF_YOUR_CHOICE]
- `Type`: choose "UDP"
- `Port count`: "1"
- `Software Description`: [EXPLAIN_WHAT_SOFTWARE_IT_IS]
- `Usage Confirmation`: I will not use this tunnel for malware, abuse, or prohibited software.
- `Public Endpoint`: choose free (you can pay if you want)
- `Assign to Agent`: Choose your agent


When it's done, head to the Playit window and you'll see a bunch of things, there is a link which is the server that you can play on and don't forget to run the server on your machine.

Launch the server:
```powershell
cargo run --release -p server
```

```powershell
cargo run --release -p client -- [YOUR_PSEUDO] [LINK_TO_THE_SERVER]
```

If you have any trouble please make an [issue](https://github.com/Alienduck/Stu_Minecraft/issues) and i'll help you and update the README.

### Architecture

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