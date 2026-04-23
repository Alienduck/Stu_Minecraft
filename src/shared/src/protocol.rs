use serde::{Deserialize, Serialize};

use crate::{block::BlockType, chunk::ChunkCoord};

pub type PlayerId = u64;

pub const DEFAULT_PORT: u16 = 25565;
pub const MAX_PACKET_SIZE: usize = 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientPacket {
    /// Initial handshake.
    Connect { name: String },
    /// Client acknowledges it has loaded the world seed and is ready.
    Ready,
    /// Player position + look, sent every frame (unreliable).
    Move {
        x: f32,
        y: f32,
        z: f32,
        yaw: f32,
        pitch: f32,
    },
    /// Request to break a block at world coordinates.
    BreakBlock { wx: i32, wy: i32, wz: i32 },
    /// Request to place a block at world coordinates.
    PlaceBlock {
        wx: i32,
        wy: i32,
        wz: i32,
        block: BlockType,
    },
    /// Keepalive reply.
    Pong { id: u32 },
    /// Graceful disconnect.
    Disconnect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerPacket {
    /// Sent immediately after Connect is accepted.
    Welcome {
        player_id: PlayerId,
        seed: u32,
        /// Server-authoritative spawn point.
        spawn_x: f32,
        spawn_y: f32,
        spawn_z: f32,
    },
    /// Another player connected.
    PlayerJoined {
        player_id: PlayerId,
        name: String,
        x: f32,
        y: f32,
        z: f32,
    },
    /// Another player disconnected.
    PlayerLeft { player_id: PlayerId },
    /// Position update for a remote player (broadcast unreliably).
    PlayerMoved {
        player_id: PlayerId,
        x: f32,
        y: f32,
        z: f32,
        yaw: f32,
        pitch: f32,
    },
    /// Authoritative single-block mutation.
    /// Clients apply this on top of their locally-generated chunk.
    BlockUpdate {
        coord: ChunkCoord,
        lx: u8,
        ly: u8,
        lz: u8,
        block: BlockType,
    },
    /// Batch of block mutations (e.g. initial delta for already-modified chunks).
    BlockBatch { updates: Vec<BlockUpdateEntry> },
    /// Time of day
    TimeUpdate { time: f32 },
    /// Keepalive — client must reply with Pong.
    Ping { id: u32 },
    /// Server rejected the connection.
    Rejected { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockUpdateEntry {
    pub coord: ChunkCoord,
    pub lx: u8,
    pub ly: u8,
    pub lz: u8,
    pub block: BlockType,
}

pub fn encode_client(pkt: &ClientPacket) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(pkt)
}

pub fn decode_client(buf: &[u8]) -> Result<ClientPacket, bincode::Error> {
    bincode::deserialize(buf)
}

pub fn encode_server(pkt: &ServerPacket) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(pkt)
}

pub fn decode_server(buf: &[u8]) -> Result<ServerPacket, bincode::Error> {
    bincode::deserialize(buf)
}
