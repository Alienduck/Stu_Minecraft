// ---------------------------------------------------------------------------
// Stu Minecraft — Headless UDP Server
//
// see (https://en.wikipedia.org/wiki/User_Datagram_Protocol)
// Responsibilities:
//   • Accept player connections over UDP
//   • Send Welcome{seed} so clients generate terrain locally
//   • Maintain the authoritative list of block mutations (deltas over
//     the procedural baseline)
//   • Broadcast BlockUpdate to all connected clients when a block changes
//   • Broadcast PlayerMoved for remote player positions
//   • Periodic Ping keepalive; drop silent clients
// ---------------------------------------------------------------------------

use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

use shared::{
    block::BlockType,
    chunk::{CHUNK_HEIGHT, CHUNK_SIZE, ChunkCoord},
    generator::TerrainGenerator,
    protocol::{
        BlockUpdateEntry, ClientPacket, DEFAULT_PORT, MAX_PACKET_SIZE, PlayerId, ServerPacket,
        decode_client, encode_server,
    },
};

const SEED: u32 = 42;
const KEEPALIVE_EVERY: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(15);
const TICK_SLEEP: Duration = Duration::from_millis(16); // ~60 Hz

struct ConnectedPlayer {
    id: PlayerId,
    name: String,
    addr: SocketAddr,
    x: f32,
    y: f32,
    z: f32,
    yaw: f32,
    pitch: f32,
    last_seen: Instant,
    ping_id: u32,
    ping_sent: Option<Instant>,
    ready: bool,
}

/// Key: (chunk_coord, lx, ly, lz)  Value: BlockType
type BlockDeltas = HashMap<(ChunkCoord, u8, u8, u8), BlockType>;

struct World {
    seed: u32,
    generator: TerrainGenerator,
    deltas: BlockDeltas,
}

impl World {
    fn new(seed: u32) -> Self {
        Self {
            seed,
            generator: TerrainGenerator::new(seed),
            deltas: HashMap::new(),
        }
    }

    fn apply_break(&mut self, wx: i32, wy: i32, wz: i32) -> Option<BlockUpdateEntry> {
        self.apply_set(wx, wy, wz, BlockType::Air)
    }

    fn apply_place(
        &mut self,
        wx: i32,
        wy: i32,
        wz: i32,
        block: BlockType,
    ) -> Option<BlockUpdateEntry> {
        self.apply_set(wx, wy, wz, block)
    }

    fn apply_set(
        &mut self,
        wx: i32,
        wy: i32,
        wz: i32,
        block: BlockType,
    ) -> Option<BlockUpdateEntry> {
        if wy < 0 || wy >= CHUNK_HEIGHT as i32 {
            return None;
        }
        let coord = ChunkCoord {
            x: wx.div_euclid(CHUNK_SIZE as i32),
            z: wz.div_euclid(CHUNK_SIZE as i32),
        };
        let lx = wx.rem_euclid(CHUNK_SIZE as i32) as u8;
        let ly = wy as u8;
        let lz = wz.rem_euclid(CHUNK_SIZE as i32) as u8;

        self.deltas.insert((coord, lx, ly, lz), block);

        Some(BlockUpdateEntry {
            coord,
            lx,
            ly,
            lz,
            block,
        })
    }

    /// Returns all current deltas as a BlockBatch packet for a newly-ready client.
    fn initial_batch(&self) -> ServerPacket {
        let updates: Vec<BlockUpdateEntry> = self
            .deltas
            .iter()
            .map(|(&(coord, lx, ly, lz), &block)| BlockUpdateEntry {
                coord,
                lx,
                ly,
                lz,
                block,
            })
            .collect();
        ServerPacket::BlockBatch { updates }
    }
}

struct Server {
    socket: UdpSocket,
    world: World,
    start_time: Instant,
    last_time_sync: Instant,
    players: HashMap<SocketAddr, ConnectedPlayer>,
    next_id: PlayerId,
    last_ping: Instant,
    ping_counter: u32,
    buf: [u8; MAX_PACKET_SIZE],
}

impl Server {
    fn new(addr: &str) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        println!("[server] listening on {addr} (seed={SEED})");
        Ok(Self {
            socket,
            world: World::new(SEED),
            start_time: Instant::now(),
            last_time_sync: Instant::now(),
            players: HashMap::new(),
            next_id: 1,
            last_ping: Instant::now(),
            ping_counter: 0,
            buf: [0u8; MAX_PACKET_SIZE],
        })
    }

    fn run(&mut self) {
        loop {
            self.recv_all();
            self.keepalive();
            self.sync_time();
            self.drop_timed_out();
            std::thread::sleep(TICK_SLEEP);
        }
    }

    fn sync_time(&mut self) {
        if self.last_time_sync.elapsed() >= Duration::from_secs(1) {
            self.last_time_sync = Instant::now();
            let time = self.start_time.elapsed().as_secs_f32();
            self.broadcast_all(&ServerPacket::TimeUpdate { time });
        }
    }

    fn recv_all(&mut self) {
        loop {
            match self.socket.recv_from(&mut self.buf) {
                Ok((n, addr)) => {
                    let data = self.buf[..n].to_vec();
                    self.handle_datagram(addr, &data);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                // Avoid useless recv error
                Err(ref e) if e.kind() == std::io::ErrorKind::ConnectionReset => break,
                Err(e) => eprintln!("[server] recv error: {e}"),
            }
        }
    }

    fn handle_datagram(&mut self, addr: SocketAddr, data: &[u8]) {
        let pkt = match decode_client(data) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[server] bad packet from {addr}: {e}");
                return;
            }
        };

        // Update last_seen for known clients
        if let Some(p) = self.players.get_mut(&addr) {
            p.last_seen = Instant::now();
        }

        match pkt {
            ClientPacket::Connect { name } => self.on_connect(addr, name),
            ClientPacket::Ready => self.on_ready(addr),
            ClientPacket::Move {
                x,
                y,
                z,
                yaw,
                pitch,
            } => {
                self.on_move(addr, x, y, z, yaw, pitch);
            }
            ClientPacket::BreakBlock { wx, wy, wz } => {
                self.on_break(addr, wx, wy, wz);
            }
            ClientPacket::PlaceBlock { wx, wy, wz, block } => {
                self.on_place(addr, wx, wy, wz, block);
            }
            ClientPacket::ChatMessage { text } => {
                self.text(addr, text);
            }
            ClientPacket::Pong { id } => self.on_pong(addr, id),
            ClientPacket::Disconnect => self.on_disconnect(addr),
        }
    }

    fn on_connect(&mut self, addr: SocketAddr, name: String) {
        if self.players.contains_key(&addr) {
            // Duplicate connect — re-send welcome (client may have lost the first)
            if let Some(p) = self.players.get(&addr) {
                let welcome = ServerPacket::Welcome {
                    player_id: p.id,
                    seed: self.world.seed,
                    spawn_x: p.x,
                    spawn_y: p.y,
                    spawn_z: p.z,
                };
                self.send_to(addr, &welcome);
            }
            return;
        }

        let id = self.next_id;
        self.next_id += 1;

        println!("[server] player '{name}' connected from {addr} (id={id})");

        let spawn = (0.0_f32, 80.0_f32, 0.0_f32);

        let player = ConnectedPlayer {
            id,
            name: name.clone(),
            addr,
            x: spawn.0,
            y: spawn.1,
            z: spawn.2,
            yaw: 0.0,
            pitch: 0.0,
            last_seen: Instant::now(),
            ping_id: 0,
            ping_sent: None,
            ready: false,
        };

        // Announce to existing players, TODO: could be useful with UI
        let joined = ServerPacket::PlayerJoined {
            player_id: id,
            name: name.clone(),
            x: spawn.0,
            y: spawn.1,
            z: spawn.2,
        };
        self.broadcast_except(addr, &joined);

        self.players.insert(addr, player);

        // Welcome the new player
        let welcome = ServerPacket::Welcome {
            player_id: id,
            seed: self.world.seed,
            spawn_x: spawn.0,
            spawn_y: spawn.1,
            spawn_z: spawn.2,
        };
        self.send_to(addr, &welcome);

        // Also tell the new player about all existing players
        let others: Vec<(PlayerId, String, f32, f32, f32)> = self
            .players
            .values()
            .filter(|p| p.addr != addr)
            .map(|p| (p.id, p.name.clone(), p.x, p.y, p.z))
            .collect();
        for (other_id, other_name, ox, oy, oz) in others {
            let pkt = ServerPacket::PlayerJoined {
                player_id: other_id,
                name: other_name,
                x: ox,
                y: oy,
                z: oz,
            };
            self.send_to(addr, &pkt);
        }
    }

    fn on_ready(&mut self, addr: SocketAddr) {
        let Some(player) = self.players.get_mut(&addr) else {
            return;
        };
        if player.ready {
            return;
        }
        player.ready = true;
        let id = player.id;
        println!("[server] player {id} is ready");

        // Send the full delta batch so the client is up-to-date
        let batch = self.world.initial_batch();
        self.send_to(addr, &batch);
    }

    fn on_move(&mut self, addr: SocketAddr, x: f32, y: f32, z: f32, yaw: f32, pitch: f32) {
        let Some(player) = self.players.get_mut(&addr) else {
            return;
        };
        player.x = x;
        player.y = y;
        player.z = z;
        player.yaw = yaw;
        player.pitch = pitch;
        let id = player.id;

        let pkt = ServerPacket::PlayerMoved {
            player_id: id,
            x,
            y,
            z,
            yaw,
            pitch,
        };
        self.broadcast_except(addr, &pkt);
    }

    fn on_break(&mut self, _addr: SocketAddr, wx: i32, wy: i32, wz: i32) {
        let Some(entry) = self.world.apply_break(wx, wy, wz) else {
            return;
        };
        println!("[server] block broken at ({wx},{wy},{wz})");
        let pkt = ServerPacket::BlockUpdate {
            coord: entry.coord,
            lx: entry.lx,
            ly: entry.ly,
            lz: entry.lz,
            block: entry.block,
        };
        self.broadcast_all(&pkt);
    }

    fn on_place(&mut self, _addr: SocketAddr, wx: i32, wy: i32, wz: i32, block: BlockType) {
        let Some(entry) = self.world.apply_place(wx, wy, wz, block) else {
            return;
        };
        println!("[server] block placed {block:?} at ({wx},{wy},{wz})");
        let pkt = ServerPacket::BlockUpdate {
            coord: entry.coord,
            lx: entry.lx,
            ly: entry.ly,
            lz: entry.lz,
            block: entry.block,
        };
        self.broadcast_all(&pkt);
    }

    fn text(&mut self, addr: SocketAddr, message: String) {
        let client_id = self.players.get(&addr);
        let Some(client) = client_id else {
            return;
        };
        let client_name = &client.name;
        let pkt = ServerPacket::ChatMessage {
            sender_name: client_name.into(),
            message,
        };
        self.broadcast_all(&pkt);
    }

    fn on_pong(&mut self, addr: SocketAddr, id: u32) {
        if let Some(p) = self.players.get_mut(&addr) {
            if p.ping_id == id {
                p.ping_sent = None;
            }
        }
    }

    fn on_disconnect(&mut self, addr: SocketAddr) {
        if let Some(player) = self.players.remove(&addr) {
            println!("[server] player '{}' disconnected", player.name);
            let pkt = ServerPacket::PlayerLeft {
                player_id: player.id,
            };
            self.broadcast_all(&pkt);
        }
    }

    fn keepalive(&mut self) {
        if self.last_ping.elapsed() < KEEPALIVE_EVERY {
            return;
        }
        self.last_ping = Instant::now();
        self.ping_counter += 1;
        let id = self.ping_counter;

        let addrs: Vec<SocketAddr> = self.players.keys().copied().collect();
        for addr in addrs {
            if let Some(p) = self.players.get_mut(&addr) {
                p.ping_id = id;
                p.ping_sent = Some(Instant::now());
            }
            self.send_to(addr, &ServerPacket::Ping { id });
        }
    }

    fn drop_timed_out(&mut self) {
        let timed_out: Vec<SocketAddr> = self
            .players
            .values()
            .filter(|p| p.last_seen.elapsed() > CLIENT_TIMEOUT)
            .map(|p| p.addr)
            .collect();

        for addr in timed_out {
            if let Some(player) = self.players.remove(&addr) {
                println!("[server] player '{}' timed out", player.name);
                let pkt = ServerPacket::PlayerLeft {
                    player_id: player.id,
                };
                self.broadcast_all(&pkt);
            }
        }
    }

    fn send_to(&self, addr: SocketAddr, pkt: &ServerPacket) {
        match encode_server(pkt) {
            Ok(data) => {
                if let Err(e) = self.socket.send_to(&data, addr) {
                    eprintln!("[server] send error to {addr}: {e}");
                }
            }
            Err(e) => eprintln!("[server] encode error: {e}"),
        }
    }

    fn broadcast_all(&self, pkt: &ServerPacket) {
        let addrs: Vec<SocketAddr> = self.players.keys().copied().collect();
        for addr in addrs {
            self.send_to(addr, pkt);
        }
    }

    fn broadcast_except(&self, exclude: SocketAddr, pkt: &ServerPacket) {
        let addrs: Vec<SocketAddr> = self
            .players
            .keys()
            .copied()
            .filter(|&a| a != exclude)
            .collect();
        for addr in addrs {
            self.send_to(addr, pkt);
        }
    }
}

fn main() {
    let addr = format!("0.0.0.0:{DEFAULT_PORT}");
    let mut server = Server::new(&addr).expect("failed to bind socket");
    server.run();
}
