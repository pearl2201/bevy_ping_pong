use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::prelude::*;
use clap::Parser;
use serde::{Deserialize, Serialize};

use bevy_replicon::{
    prelude::*,
    renet::{
        transport::{
            ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport,
            ServerAuthentication, ServerConfig,
        },
        ClientId, ConnectionConfig, ServerEvent,
    },
};

pub const PORT: u16 = 5000;
pub const PROTOCOL_ID: u64 = 0;

pub struct SimpleBoxPlugin;

impl Plugin for SimpleBoxPlugin {
    fn build(&self, app: &mut App) {
        app.replicate::<PlayerPosition>()
            .replicate::<PlayerColor>()
            .add_client_event::<MoveDirection>(EventType::Ordered)
            .add_systems(Startup, Self::init_system)
            .add_systems(
                Update,
                (
                    Self::movement_system.run_if(has_authority()), // Runs only on the server or a single player.
                    Self::server_event_system.run_if(resource_exists::<RenetServer>()), // Runs only on the server.
                    (Self::draw_boxes_system, Self::input_system),
                ),
            );
    }
}

impl SimpleBoxPlugin {
    fn init_system(mut commands: Commands) {
        commands.spawn(Camera2dBundle::default());
    }

    /// Logs server events and spawns a new player whenever a client connects.
    fn server_event_system(mut commands: Commands, mut server_event: EventReader<ServerEvent>) {
        for event in server_event.read() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    info!("player: {client_id} Connected");
                    // Generate pseudo random color from client id.
                    let r = ((client_id.raw() % 23) as f32) / 23.0;
                    let g = ((client_id.raw() % 27) as f32) / 27.0;
                    let b = ((client_id.raw() % 39) as f32) / 39.0;
                    commands.spawn(PlayerBundle::new(
                        *client_id,
                        Vec2::ZERO,
                        Color::rgb(r, g, b),
                    ));
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    info!("client {client_id} disconnected: {reason}");
                }
            }
        }
    }

    fn draw_boxes_system(mut gizmos: Gizmos, players: Query<(&PlayerPosition, &PlayerColor)>) {
        for (position, color) in &players {
            gizmos.rect(
                Vec3::new(position.x, position.y, 0.0),
                Quat::IDENTITY,
                Vec2::ONE * 50.0,
                color.0,
            );
        }
    }

    /// Reads player inputs and sends [`MoveCommandEvents`]
    fn input_system(mut move_events: EventWriter<MoveDirection>, input: Res<Input<KeyCode>>) {
        let mut direction = Vec2::ZERO;
        if input.pressed(KeyCode::Right) {
            direction.x += 1.0;
        }
        if input.pressed(KeyCode::Left) {
            direction.x -= 1.0;
        }
        if input.pressed(KeyCode::Up) {
            direction.y += 1.0;
        }
        if input.pressed(KeyCode::Down) {
            direction.y -= 1.0;
        }
        if direction != Vec2::ZERO {
            move_events.send(MoveDirection(direction.normalize_or_zero()));
        }
    }

    /// Mutates [`PlayerPosition`] based on [`MoveCommandEvents`].
    ///
    /// Fast-paced games usually you don't want to wait until server send a position back because of the latency.
    /// But this example just demonstrates simple replication concept.
    fn movement_system(
        time: Res<Time>,
        mut move_events: EventReader<FromClient<MoveDirection>>,
        mut players: Query<(&Player, &mut PlayerPosition)>,
    ) {
        const MOVE_SPEED: f32 = 300.0;
        for FromClient { client_id, event } in move_events.read() {
            info!("received event {event:?} from client {client_id}");
            for (player, mut position) in &mut players {
                if *client_id == player.0 {
                    **position += event.0 * time.delta_seconds() * MOVE_SPEED;
                }
            }
        }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    position: PlayerPosition,
    color: PlayerColor,
    replication: Replication,
}

impl PlayerBundle {
    pub fn new(client_id: ClientId, position: Vec2, color: Color) -> Self {
        Self {
            player: Player(client_id),
            position: PlayerPosition(position),
            color: PlayerColor(color),
            replication: Replication,
        }
    }
}

/// Contains the client ID of the player.
#[derive(Component, Serialize, Deserialize)]
struct Player(ClientId);

#[derive(Component, Deserialize, Serialize, Deref, DerefMut)]
struct PlayerPosition(Vec2);

#[derive(Component, Deserialize, Serialize)]
struct PlayerColor(Color);

/// A movement event for the controlled box.
#[derive(Debug, Default, Deserialize, Event, Serialize)]
struct MoveDirection(Vec2);
