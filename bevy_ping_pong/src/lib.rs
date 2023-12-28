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
const SCREEN_WIDTH: f32 = 1280.0;
const SCREEN_HEGIHT: f32 = 720.0;
const PADDLE_WIDTH: f32 = 100.0;
const PADDLE_HEIGHT: f32 = 300.0;
const BALL_WIDTH: f32 = 100.0;
const PADDLE_LEFT_X: f32 = SCREEN_WIDTH / 2.0 - PADDLE_WIDTH / 2.0;
const PADDLE_LEFT_Y: f32 = -(SCREEN_WIDTH / 2.0 - PADDLE_WIDTH / 2.0);
const CLAMP_MAX_PADDLE_Y: f32 = SCREEN_HEGIHT / 2.0 - PADDLE_HEIGHT / 2.0;
const CLAMP_MIN_PADDLE_Y: f32 = -CLAMP_MAX_PADDLE_Y;
const CLAMP_MAX_BALL_Y: f32 = SCREEN_HEGIHT / 2.0 - BALL_WIDTH / 2.0;
const CLAMP_MIN_BALL_Y: f32 = -CLAMP_MAX_BALL_Y;
const CLAMP_MAX_BALL_X: f32 = SCREEN_WIDTH / 2.0 - BALL_WIDTH / 2.0;
const CLAMP_MIN_BALL_X: f32 = -CLAMP_MAX_BALL_X;

pub struct PingPongPlugin;

impl Plugin for PingPongPlugin {
    fn build(&self, app: &mut App) {
        app.replicate::<PlayerPosition>()
            .replicate::<PlayerColor>()
            .add_client_event::<MoveDirection>(EventType::Ordered)
            .add_systems(Startup, Self::init_system)
            .add_systems(
                Update,
                (
                    Self::movement_system.run_if(has_authority()),
                    Self::server_event_system.run_if(resource_exists::<RenetServer>()),
                    (Self::draw_boxes_system, Self::input_system),
                ),
            );
    }
}

impl PingPongPlugin {
    fn init_system(mut commands: Commands) {
        commands.spawn(Camera2dBundle::default());
        commands.spawn(BallBundle::new(Vec2::ZERO, Color::rgb(1.0, 1.0, 1.0)));
    }

    fn movement_system() {}

    fn server_event_system() {}

    fn draw_boxes_system() {}

    fn input_system() {}
}

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    position: PlayerPosition,
    color: PlayerColor,
    replication: Replication,
    paddle: Paddle,
}

impl PlayerBundle {
    pub fn new(client_id: ClientId, position: Vec2, color: Color) -> Self {
        Self {
            player: Player(client_id),
            position: PlayerPosition(position),
            color: PlayerColor(color),
            replication: Replication,
            paddle: Paddle {},
        }
    }
}

#[derive(Bundle)]
pub struct BallBundle {
    position: PlayerPosition,
    color: PlayerColor,
    replication: Replication,
    ball: Ball,
}

impl BallBundle {
    pub fn new(position: Vec2, color: Color) -> Self {
        Self {
            position: PlayerPosition(position),
            color: PlayerColor(color),
            replication: Replication,
            ball: Ball {},
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

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;
