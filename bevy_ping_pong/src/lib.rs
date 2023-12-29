use bevy::prelude::*;

use bevy_egui::{
    egui::{self, Frame, Pos2, Ui},
    EguiContext,
};
use serde::{Deserialize, Serialize};

use bevy_replicon::{
    prelude::*,
    renet::{ClientId, ServerEvent},
};

pub const PORT: u16 = 5000;
pub const PROTOCOL_ID: u64 = 0;
const SCREEN_WIDTH: f32 = 1280.0;
const SCREEN_HEGIHT: f32 = 720.0;
const PADDLE_WIDTH: f32 = 50.0;
const PADDLE_HEIGHT: f32 = 250.0;
const BALL_WIDTH: f32 = 50.0;
const SPEED: f32 = 150.0;
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
    actor1: u64,
    actor2: u64,
    score1: u16,
    score2: u16,
    round: u16,
}

#[derive(Resource)]
pub struct LocalData {
    pub client_id: u64,
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
                actor1: 0,
                actor2: 0,
                score1: 0,
                score2: 0,
                round: 0,
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
                        .run_if(has_authority())
                        .run_if(in_state(GameState::Game)),
                    Self::server_event_system.run_if(resource_exists::<RenetServer>()),
                    Self::client_event_system.run_if(resource_exists::<RenetClient>()),
                    (Self::draw_boxes_system, Self::input_system)
                        .run_if(not(in_state(GameState::Menu)))
                        .run_if(resource_exists::<RenetClient>()),
                ),
            );
    }
}

impl PingPongPlugin {
    fn init_system(mut commands: Commands) {
        commands.spawn(Camera2dBundle {
            camera: Camera {
                order: -1,
                ..default()
            },
            ..default()
        });
        commands.insert_resource(SplashTimer(Timer::from_seconds(
            1.0 / 60.0,
            TimerMode::Repeating,
        )));
    }

    pub fn init_system_server(mut commands: Commands) {
        println!("spawn ball");
        commands.spawn(BallBundle::new(Vec2::ZERO, Color::rgb(1.0, 1.0, 1.0)));
    }

    fn client_event_system(
        mut move_events: EventReader<ServerMessage>,
        mut game_state: ResMut<NextState<GameState>>,
        mut game_data: ResMut<GameData>,
    ) {
        for event in move_events.read() {
            match event.msg {
                S2cMessage::None => {}
                S2cMessage::GameStart(actor1_id, actor2_id) => {
                    game_state.set(GameState::Game);
                    game_data.actor1 = actor1_id;
                    game_data.actor2 = actor2_id;
                }
                S2cMessage::ClientJoin(client_id, client_actor_id) => {
                    if (client_actor_id == 1) {
                        game_data.actor1 = client_id;
                    } else {
                        game_data.actor2 = client_id;
                    }
                }
                S2cMessage::RoundResult(client_actor_id) => {
                    if (client_actor_id == 1) {
                        game_data.score1 += 1;
                    } else {
                        game_data.score2 += 1;
                    }
                    game_data.round += 1;
                }
                S2cMessage::GameEnd => game_state.set(GameState::End),
            }
        }
    }
    fn movement_system(
        time: Res<Time>,
        mut game_date: ResMut<GameData>,
        mut move_events: EventReader<FromClient<MoveDirection>>,
        mut paddles: Query<(&Player, &mut PlayerPosition), (With<Paddle>, Without<Ball>)>,
        mut ball: Query<(&mut PlayerPosition, &mut PlayerSpeed), (With<Ball>, Without<Paddle>)>,
        mut nextState: ResMut<NextState<GameState>>,
        mut game_message_events: EventWriter<ToClients<ServerMessage>>,
    ) {
        const MOVE_SPEED: f32 = SPEED;
        for FromClient { client_id, event } in move_events.read() {
            for (player, mut position) in &mut paddles {
                if *client_id == player.0 {
                    let f = event.0 * 1.0 * MOVE_SPEED / 60.0;
                    **position += f;
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
        if ball_pos.y <= CLAMP_MIN_BALL_Y {
            ball_velocivy.y = -ball_velocivy.y;
        } else if ball_pos.y >= CLAMP_MAX_BALL_Y {
            ball_velocivy.y = -ball_velocivy.y;
        }

        ball_pos.y = f32::clamp(ball_pos.y, CLAMP_MIN_BALL_Y, CLAMP_MAX_BALL_Y);
        let mut is_reset: bool = false;
        if (ball_pos.x <= CLAMP_MIN_BALL_X && game_date.round < 3) {
            ball_pos.x = 0.0;
            ball_pos.y = 0.0;
            ball_velocivy.x = -ball_velocivy.x;
            game_date.round += 1;
            game_date.score1 += 1;
            game_message_events.send(ToClients {
                mode: SendMode::Broadcast,
                event: ServerMessage {
                    msg: S2cMessage::RoundResult(2),
                },
            });
            is_reset = true;
        } else if (ball_pos.x >= CLAMP_MAX_BALL_X && game_date.round < 3) {
            ball_pos.x = 0.0;
            ball_pos.y = 0.0;
            ball_velocivy.x = -ball_velocivy.x;
            game_date.round += 1;
            game_date.score2 += 1;
            game_message_events.send(ToClients {
                mode: SendMode::Broadcast,
                event: ServerMessage {
                    msg: S2cMessage::RoundResult(1),
                },
            });
            is_reset = true;
        }

        if (is_reset && game_date.round >= 3) {
            println!("game end: {}", game_date.round);
            nextState.set(GameState::End);
            game_message_events.send(ToClients {
                mode: SendMode::Broadcast,
                event: ServerMessage {
                    msg: S2cMessage::GameEnd,
                },
            });
        }

        for (_player, position) in &paddles {
            let collision = Self::intersect(
                Vec2::new(ball_pos.x, ball_pos.y),
                Vec2::new(BALL_WIDTH, BALL_WIDTH),
                Vec2::new(position.x, position.y),
                Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT),
            );
            if collision {
                ball_velocivy.x = -ball_velocivy.x;
                if ball_pos.x < 0.0 {
                    ball_velocivy.x = ball_velocivy.x.abs()
                } else {
                    ball_velocivy.x = -ball_velocivy.x.abs()
                }
            }
        }
    }

    fn intersect(center_a: Vec2, size_a: Vec2, center_b: Vec2, size_b: Vec2) -> bool {
        return center_a.x - size_a.x / 2.0 <= center_b.x + size_b.x / 2.0
            && center_a.x + size_a.x / 2.0 >= center_b.x - size_b.x / 2.0
            && center_a.y - size_a.y / 2.0 <= center_b.y + size_b.y / 2.0
            && center_a.y + size_a.y / 2.0 >= center_b.y - size_b.y / 2.0;
    }

    fn server_event_system(
        mut commands: Commands,
        mut server_event: EventReader<ServerEvent>,
        mut game_state: ResMut<NextState<GameState>>,
        mut game_data: ResMut<GameData>,
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
                    if game_data.player_count == 0 {
                        commands.spawn(PlayerBundle::new(
                            *client_id,
                            Vec2::new(-PADDLE_LEFT_X, 0.0),
                            Color::rgb(r, g, b),
                        ));
                        game_message_events.send(ToClients {
                            mode: SendMode::Broadcast,
                            event: ServerMessage {
                                msg: S2cMessage::ClientJoin(client_id.raw(), 1),
                            },
                        });
                        game_data.actor1 = client_id.raw();
                        game_data.player_count += 1;
                    } else if game_data.player_count == 1 {
                        commands.spawn(PlayerBundle::new(
                            *client_id,
                            Vec2::new(PADDLE_LEFT_X, 0.0),
                            Color::rgb(r, g, b),
                        ));

                        game_message_events.send(ToClients {
                            mode: SendMode::Broadcast,
                            event: ServerMessage {
                                msg: S2cMessage::ClientJoin(client_id.raw(), 2),
                            },
                        });
                        game_data.actor2 = client_id.raw();
                        game_data.player_count += 1;
                    }

                    if game_data.player_count == 2 {
                        game_state.set(GameState::Game);
                        game_message_events.send(ToClients {
                            mode: SendMode::Broadcast,
                            event: ServerMessage {
                                msg: S2cMessage::GameStart(game_data.actor1, game_data.actor2),
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
        mut egui_ctx: Query<&mut EguiContext>,
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

    pub fn render_gui_client(
        mut egui_ctx: Query<&mut EguiContext>,
        game_data: Res<GameData>,
        local_data: Res<LocalData>,
    ) {
        let client_id = local_data.client_id;
        let my_score = if client_id == game_data.actor1 {
            game_data.score1
        } else {
            game_data.score2
        };
        let opponent_id = if client_id == game_data.actor1 {
            game_data.actor2
        } else {
            game_data.actor1
        };

        let opponent_score = if client_id == game_data.actor1 {
            game_data.score2
        } else {
            game_data.score1
        };
        egui::CentralPanel::default().frame(Frame::none()).show(
            &egui_ctx.single_mut().get_mut(),
            |ui| {
                ui.horizontal_top(|ui| {
                    ui.vertical_centered(|ui| ui.label(format!("Round: {}/{}", game_data.round,3)))
                });

                ui.put(
                    bevy_egui::egui::Rect {
                        min: Pos2 { x: 0.0, y: 0.0 },
                        max: Pos2 { x: 250.0, y: 100.0 },
                    },
                    |ui: &mut Ui| {
                        ui.horizontal_top(|ui| ui.label(format!("Client: {client_id}")))
                            .response;
                        ui.horizontal_top(|ui| ui.label(format!("Score: {my_score}")))
                            .response
                    },
                );

                ui.put(
                    bevy_egui::egui::Rect {
                        min: Pos2 { x: 1030.0, y: 0.0 },
                        max: Pos2 {
                            x: 1280.0,
                            y: 100.0,
                        },
                    },
                    |ui: &mut Ui| {
                        ui.horizontal_top(|ui| ui.label(format!("Client: {opponent_id}")))
                            .response;
                        ui.horizontal_top(|ui| ui.label(format!("Score: {opponent_score}")))
                            .response
                    },
                );
            },
        );
        // egui::CentralPanel::default().show(egui_ctx.single_mut().get_mut(), |ui| {
        // ui.horizontal_top(|ui| {
        //     ui.vertical_centered(|ui| ui.label(format!("Round: {}", game_data.round + 1)))
        // });

        // ui.put(
        //     bevy_egui::egui::Rect {
        //         min: Pos2 { x: 0.0, y: 0.0 },
        //         max: Pos2 { x: 250.0, y: 100.0 },
        //     },
        //     |ui: &mut Ui| {
        //         ui.horizontal_top(|ui| ui.label(format!("Client: {opponent_id}")))
        //             .response;
        //         ui.horizontal_top(|ui| ui.label(format!("Score: {opponent_score}")))
        //             .response
        //     },
        // );

        // ui.put(
        //     bevy_egui::egui::Rect {
        //         min: Pos2 { x: 1030.0, y: 0.0 },
        //         max: Pos2 {
        //             x: 1280.0,
        //             y: 100.0,
        //         },
        //     },
        //     |ui: &mut Ui| {
        //         ui.horizontal_top(|ui| ui.label(format!("Client: {client_id}")))
        //             .response;
        //         ui.horizontal_top(|ui| ui.label(format!("Score: {my_score}")))
        //             .response
        //     },
        // );
        // });
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
            speed: PlayerSpeed(Vec2::new(SPEED, SPEED)),
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
    msg: S2cMessage,
}
#[derive(Debug, Default, Deserialize, Serialize)]
pub enum S2cMessage {
    #[default]
    None,
    GameStart(u64, u64),
    ClientJoin(u64, i32),
    RoundResult(u64),
    GameEnd,
}

#[derive(Resource, Deref, DerefMut)]
struct SplashTimer(Timer);
