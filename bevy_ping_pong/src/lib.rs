use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
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
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Menu,
    Game,
    End,
}
#[derive(Resource, Default)]
pub struct GameData {
    player_count: usize,
    score1: u16,
    score2: u16,
    round: u16,
}

pub struct PingPongPlugin;

impl Plugin for PingPongPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .replicate::<PlayerPosition>()
            .replicate::<PlayerColor>()
            .replicate::<Ball>()
            .replicate::<Paddle>()
            .insert_resource::<GameData>(GameData {
                player_count: 0,
                score1: 0,
                score2: 0,
                round: 1,
            })
            .add_client_event::<MoveDirection>(EventType::Ordered)
            .add_server_event::<ServerMessage>(EventType::Ordered)
            .add_systems(Startup, (Self::init_system,))
            .add_systems(
                OnEnter(GameState::Game),
                (Self::notify_game_state).run_if(resource_exists::<RenetServer>()),
            )
            .add_systems(
                Update,
                (
                    Self::movement_system
                        .run_if(resource_exists::<RenetServer>())
                        .run_if(in_state(GameState::Game)),
                    Self::server_event_system.run_if(resource_exists::<RenetServer>()),
                    Self::client_event_system.run_if(resource_exists::<RenetClient>()),
                    (Self::draw_boxes_system, Self::input_system)
                        .run_if(in_state(GameState::Game))
                        .run_if(resource_exists::<RenetClient>()),
                ),
            );
    }
}

impl PingPongPlugin {
    fn init_system(mut commands: Commands) {
        commands.spawn(Camera2dBundle::default());
    }

    pub fn init_system_server(mut commands: Commands) {
        println!("spawn ball");
        commands.spawn(BallBundle::new(Vec2::ZERO, Color::rgb(1.0, 1.0, 1.0)));
    }

    fn client_event_system(
        mut move_events: EventReader<ServerMessage>,
        mut game_state: ResMut<NextState<GameState>>,
    ) {
        for event in move_events.read() {
            println!("update client_event_system");
            match event.msg {
                S2C_Message::GameStart => game_state.set(GameState::Game),
            }
        }
    }
    fn movement_system(
        time: Res<Time>,
        mut move_events: EventReader<FromClient<MoveDirection>>,
        mut paddles: Query<(&Player, &mut PlayerPosition), (With<Paddle>, Without<Ball>)>,
        mut ball: Query<(&mut PlayerPosition, &mut PlayerSpeed), (With<Ball>, Without<Paddle>)>,
    ) {
        const MOVE_SPEED: f32 = 100.0;
        for FromClient { client_id, event } in move_events.read() {
            info!("received event {event:?} from client {client_id}");
            for (player, mut position) in &mut paddles {
                if *client_id == player.0 {
                    **position += event.0 * time.delta_seconds() * MOVE_SPEED;
                    position.y = position.y.clamp(CLAMP_MIN_PADDLE_Y, CLAMP_MAX_PADDLE_Y);
                }
            }
        }

        let (mut ball_pos, mut ball_velocivy) = ball.single_mut();
        ball_pos.x = ball_pos.x + ball_velocivy.x * time.delta_seconds();
        ball_pos.y = ball_pos.y + ball_velocivy.y * time.delta_seconds();
        // if (ball_pos.translation.x <= CLAMP_MIN_BALL_X || ball_pos.translation.x >= CLAMP_MAX_BALL_X)
        // {
        //     game_state.set(GameState::End);
        // }
        if (ball_pos.y <= CLAMP_MIN_BALL_Y) {
            ball_velocivy.y = -ball_velocivy.y;
        } else if (ball_pos.y >= CLAMP_MAX_BALL_Y) {
            ball_velocivy.y = -ball_velocivy.y;
        }
        ball_pos.x = f32::clamp(ball_pos.x, CLAMP_MIN_BALL_X, CLAMP_MAX_BALL_X);
        ball_pos.y = f32::clamp(ball_pos.y, CLAMP_MIN_BALL_Y, CLAMP_MAX_BALL_Y);

        for (player, mut position) in &paddles {
            let collision = collide(
                Vec3::new(ball_pos.x, ball_pos.y, 0.0),
                Vec2::new(BALL_WIDTH, BALL_WIDTH),
                Vec3::new(position.x, position.y, 0.0),
                Vec2::new(BALL_WIDTH, BALL_WIDTH),
            );
            if let Some(collision) = collision {
                // Sends a collision event so that other systems can react to the collision
                //collision_events.send_default();
                // reflect the ball when it collides
                let mut reflect_x = false;
                let mut reflect_y = false;

                // only reflect if the ball's velocity is going in the opposite direction of the
                // collision
                match collision {
                    Collision::Left => reflect_x = ball_velocivy.x > 0.0,
                    Collision::Right => reflect_x = ball_velocivy.x < 0.0,
                    Collision::Top => reflect_y = ball_velocivy.y < 0.0,
                    Collision::Bottom => reflect_y = ball_velocivy.y > 0.0,
                    Collision::Inside => { /* do nothing */ }
                }

                // reflect velocity on the x-axis if we hit something on the x-axis
                if reflect_x {
                    ball_velocivy.x = -ball_velocivy.x;
                }

                // reflect velocity on the y-axis if we hit something on the y-axis
                if reflect_y {
                    ball_velocivy.y = -ball_velocivy.y;
                }
            }
        }
    }

    fn server_event_system(
        mut commands: Commands,
        mut server_event: EventReader<ServerEvent>,
        mut game_state: ResMut<NextState<GameState>>,
        app_state: Res<State<GameState>>,
        mut gameData: ResMut<GameData>,
        mut game_message_events: EventWriter<ToClients<ServerMessage>>,
    ) {
        for event in server_event.read() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    info!("player: {client_id} Connected");
                    // Generate pseudo random color from client id.
                    let r = ((client_id.raw() % 23) as f32) / 23.0;
                    let g = ((client_id.raw() % 27) as f32) / 27.0;
                    let b = ((client_id.raw() % 39) as f32) / 39.0;
                    if (gameData.player_count == 0) {
                        commands.spawn(PlayerBundle::new(
                            *client_id,
                            Vec2::new(-PADDLE_LEFT_X, 0.0),
                            Color::rgb(r, g, b),
                        ));
                    } else {
                        commands.spawn(PlayerBundle::new(
                            *client_id,
                            Vec2::new(PADDLE_LEFT_X, 0.0),
                            Color::rgb(r, g, b),
                        ));
                    }

                    gameData.player_count += 1;

                    if (gameData.player_count >= 2) {
                        game_state.set(GameState::Game);
                        game_message_events.send(ToClients {
                            mode: SendMode::Broadcast,
                            event: ServerMessage {
                                msg: S2C_Message::GameStart,
                            },
                        });
                        println!("send gamestart");
                    }
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    info!("client {client_id} disconnected: {reason}");
                }
            }
        }
    }

    fn notify_game_state() {}
    fn draw_boxes_system(
        mut gizmos: Gizmos,
        players: Query<(&PlayerPosition, &PlayerColor), With<Paddle>>,
        ball: Query<(&PlayerPosition, &PlayerColor), With<Ball>>,
    ) {
        for (position, color) in &players {
            gizmos.rect(
                Vec3::new(position.x, position.y, 0.0),
                Quat::IDENTITY,
                Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT),
                color.0,
            );
        }
        for (ball_pos, ball_color) in &ball {
            gizmos.rect(
                Vec3::new(ball_pos.x, ball_pos.y, 0.0),
                Quat::IDENTITY,
                Vec2::new(BALL_WIDTH, BALL_WIDTH),
                ball_color.0,
            )
        }
    }

    fn input_system(mut move_events: EventWriter<MoveDirection>, input: Res<Input<KeyCode>>) {
        let mut direction = Vec2::ZERO;
        // if input.pressed(KeyCode::Right) {
        //     direction.x += 1.0;
        // }
        // if input.pressed(KeyCode::Left) {
        //     direction.x -= 1.0;
        // }
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
    speed: PlayerSpeed,
    color: PlayerColor,
    replication: Replication,
    ball: Ball,
}

impl BallBundle {
    pub fn new(position: Vec2, color: Color) -> Self {
        Self {
            position: PlayerPosition(position),
            speed: PlayerSpeed(Vec2::new(100.0, 100.0)),
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

#[derive(Component, Deserialize, Serialize, Deref, DerefMut)]
struct PlayerSpeed(Vec2);

/// A movement event for the controlled box.
#[derive(Debug, Default, Deserialize, Event, Serialize)]
struct MoveDirection(Vec2);

#[derive(Component, Serialize, Deserialize)]
struct Paddle;

#[derive(Component, Serialize, Deserialize)]
struct Ball;

#[derive(Debug, Default, Deserialize, Event, Serialize)]
pub struct ServerMessage {
    msg: S2C_Message,
}
#[derive(Debug, Default, Deserialize, Serialize)]
pub enum S2C_Message {
    #[default]
    GameStart,
}
