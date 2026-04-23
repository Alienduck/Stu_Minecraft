use bevy::prelude::*;
use shared::protocol::{
    ClientPacket, DEFAULT_PORT, MAX_PACKET_SIZE, ServerPacket, decode_server, encode_client,
};
use std::{
    net::UdpSocket,
    sync::{
        Mutex,
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
    thread,
    time::Duration,
};

#[derive(Resource)]
pub struct NetSender(pub Mutex<Sender<ClientPacket>>);

#[derive(Resource)]
pub struct NetReceiver(pub Mutex<Receiver<ServerPacket>>);

#[derive(Debug, Clone, Message)]
pub struct EvWelcome {
    pub player_id: u64,
    pub seed: u32,
    pub spawn: Vec3,
}

#[derive(Message, Debug, Clone)]
pub struct EvBlockUpdate {
    pub coord: shared::chunk::ChunkCoord,
    pub lx: u8,
    pub ly: u8,
    pub lz: u8,
    pub block: shared::block::BlockType,
}

#[derive(Message, Debug, Clone)]
pub struct EvPlayerJoined {
    pub player_id: u64,
    pub name: String,
    pub pos: Vec3,
}

#[derive(Message, Debug, Clone)]
pub struct EvPlayerLeft {
    pub player_id: u64,
}

#[derive(Message, Debug, Clone)]
pub struct EvPlayerMoved {
    pub player_id: u64,
    pub pos: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Message, Debug, Clone)]
pub struct EvTimeUpdate {
    pub time: f32,
}

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        let addr_arg = std::env::args()
            .nth(2)
            .unwrap_or_else(|| "127.0.0.1".into());

        let server_addr = if addr_arg.contains(':') {
            addr_arg
        } else {
            format!("{addr_arg}:{DEFAULT_PORT}")
        };
        let (out_tx, out_rx) = mpsc::channel::<ClientPacket>();
        let (in_tx, in_rx) = mpsc::channel::<ServerPacket>();

        spawn_net_thread(server_addr, out_rx, in_tx);

        app.insert_resource(NetSender(Mutex::new(out_tx)))
            .insert_resource(NetReceiver(Mutex::new(in_rx)))
            .add_message::<EvWelcome>()
            .add_message::<EvBlockUpdate>()
            .add_message::<EvPlayerJoined>()
            .add_message::<EvPlayerLeft>()
            .add_message::<EvPlayerMoved>()
            .add_message::<EvTimeUpdate>()
            .add_systems(Startup, send_connect)
            .add_systems(PreUpdate, dispatch_incoming);
    }
}

fn spawn_net_thread(
    server_addr: String,
    out_rx: Receiver<ClientPacket>,
    in_tx: Sender<ServerPacket>,
) {
    thread::spawn(move || {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("UDP bind failed");
        socket.set_nonblocking(true).unwrap();
        socket.connect(&server_addr).expect("UDP connect failed");

        let mut buf = [0u8; MAX_PACKET_SIZE];
        loop {
            loop {
                match out_rx.try_recv() {
                    Ok(pkt) => {
                        if let Ok(data) = encode_client(&pkt) {
                            let _ = socket.send(&data);
                        }
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return,
                }
            }
            loop {
                match socket.recv(&mut buf) {
                    Ok(n) => {
                        if let Ok(pkt) = decode_server(&buf[..n]) {
                            if in_tx.send(pkt).is_err() {
                                return;
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(_) => break,
                }
            }
            thread::sleep(Duration::from_millis(4));
        }
    });
}

fn send_connect(sender: Res<NetSender>) {
    let name = std::env::args().nth(1).unwrap_or_else(|| "Player".into());
    let _ = sender
        .0
        .lock()
        .unwrap()
        .send(ClientPacket::Connect { name });
}

fn dispatch_incoming(
    receiver: Res<NetReceiver>,
    mut ev_welcome: MessageWriter<EvWelcome>,
    mut ev_block: MessageWriter<EvBlockUpdate>,
    mut ev_joined: MessageWriter<EvPlayerJoined>,
    mut ev_left: MessageWriter<EvPlayerLeft>,
    mut ev_moved: MessageWriter<EvPlayerMoved>,
    mut ev_time: MessageWriter<EvTimeUpdate>,
    sender: Res<NetSender>,
) {
    let rx = receiver.0.lock().unwrap();
    loop {
        match rx.try_recv() {
            Ok(pkt) => handle_server_packet(
                pkt,
                &mut ev_welcome,
                &mut ev_block,
                &mut ev_joined,
                &mut ev_left,
                &mut ev_moved,
                &mut ev_time,
                &sender,
            ),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
        }
    }
}

fn handle_server_packet(
    pkt: ServerPacket,
    ev_welcome: &mut MessageWriter<EvWelcome>,
    ev_block: &mut MessageWriter<EvBlockUpdate>,
    ev_joined: &mut MessageWriter<EvPlayerJoined>,
    ev_left: &mut MessageWriter<EvPlayerLeft>,
    ev_moved: &mut MessageWriter<EvPlayerMoved>,
    ev_time: &mut MessageWriter<EvTimeUpdate>,
    sender: &NetSender,
) {
    match pkt {
        ServerPacket::Welcome {
            player_id,
            seed,
            spawn_x,
            spawn_y,
            spawn_z,
        } => {
            ev_welcome.write(EvWelcome {
                player_id,
                seed,
                spawn: Vec3::new(spawn_x, spawn_y, spawn_z),
            });
        }
        ServerPacket::BlockUpdate {
            coord,
            lx,
            ly,
            lz,
            block,
        } => {
            ev_block.write(EvBlockUpdate {
                coord,
                lx,
                ly,
                lz,
                block,
            });
        }
        ServerPacket::BlockBatch { updates } => {
            for u in updates {
                ev_block.write(EvBlockUpdate {
                    coord: u.coord,
                    lx: u.lx,
                    ly: u.ly,
                    lz: u.lz,
                    block: u.block,
                });
            }
        }
        ServerPacket::PlayerJoined {
            player_id,
            name,
            x,
            y,
            z,
        } => {
            ev_joined.write(EvPlayerJoined {
                player_id,
                name,
                pos: Vec3::new(x, y, z),
            });
        }
        ServerPacket::PlayerLeft { player_id } => {
            ev_left.write(EvPlayerLeft { player_id });
        }
        ServerPacket::PlayerMoved {
            player_id,
            x,
            y,
            z,
            yaw,
            pitch,
        } => {
            ev_moved.write(EvPlayerMoved {
                player_id,
                pos: Vec3::new(x, y, z),
                yaw,
                pitch,
            });
        }
        ServerPacket::TimeUpdate { time } => {
            ev_time.write(EvTimeUpdate { time });
        }
        // Was used for testing and now for keepalive 🗿
        ServerPacket::Ping { id } => {
            let _ = sender.0.lock().unwrap().send(ClientPacket::Pong { id });
        }
        ServerPacket::Rejected { .. } => {}
    }
}
