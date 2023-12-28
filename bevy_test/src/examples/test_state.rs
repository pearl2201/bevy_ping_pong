use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use rand::Rng;
// Enum that will be used as a global state for the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Splash,
    Menu,
    Game,
    End,
}

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Paddle;
#[derive(Resource, Default)]
struct Game {
    speed: Vec3,
}
#[derive(Resource, Deref, DerefMut)]
struct SplashTimer(Timer);

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

fn main() {
    // let speed_x: f32 = rand::thread_rng().gen_range(-100.0..-50.0);
    // let speed_y: f32 = rand::thread_rng().gen_range(-100.0..-50.0);
    let speed_x: f32 = -100.0;
    let speed_y: f32 = -100.0;
    rand::thread_rng().gen_range(-100..-50);
    App::new()
    .add_event::<CollisionEvent>()
        .add_plugins(DefaultPlugins)
        .add_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            ((mouse_click_system, ball_move_system).run_if(in_state(GameState::Game))),
        )
        .add_systems(
            Update,
            (update_change_state).run_if(in_state(GameState::Splash)),
        )
        .insert_resource::<Game>(Game {
            speed: Vec3::new(speed_x, speed_y, 0.0),
        })
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        (SpriteBundle {
            transform: Transform {
                // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                // This is used to determine the order of our sprites
                translation: Vec3::new(0.0, 0.0, 0.0),
                // The z-scale of 2D objects must always be 1.0,
                // or their ordering will be affected in surprising ways.
                // See https://github.com/bevyengine/bevy/issues/4149
                scale: Vec3::new(100.0, 100.0, 100.0),
                ..default()
            },
            ..default()
        }),
        Ball,
    ));

    commands.spawn((
        (SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(0.5, 0.5, 0.5, 0.7),
                ..default()
            },
            transform: Transform {
                // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                // This is used to determine the order of our sprites
                translation: Vec3::new(PADDLE_LEFT_X, 0.0, 0.0),
                // The z-scale of 2D objects must always be 1.0,
                // or their ordering will be affected in surprising ways.
                // See https://github.com/bevyengine/bevy/issues/4149
                scale: Vec3::new(PADDLE_WIDTH, PADDLE_HEIGHT, 100.0),
                ..default()
            },
            ..default()
        }),
        Paddle,
    ));

    commands.spawn((
        (SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(0.5, 0.5, 0.5, 0.7),
                ..default()
            },
            transform: Transform {
                // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                // This is used to determine the order of our sprites
                translation: Vec3::new(-PADDLE_LEFT_X, 0.0, 0.0),
                // The z-scale of 2D objects must always be 1.0,
                // or their ordering will be affected in surprising ways.
                // See https://github.com/bevyengine/bevy/issues/4149
                scale: Vec3::new(PADDLE_WIDTH, PADDLE_HEIGHT, 100.0),
                ..default()
            },
            ..default()
        }),
        Paddle,
    ));
    commands.insert_resource(SplashTimer(Timer::from_seconds(1.0, TimerMode::Once)));
}

fn ball_move_system(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut game_state: ResMut<NextState<GameState>>,
    mut transforms: Query<(&mut Transform), With<Ball>>,
    mut paddle_query: Query<(&mut Transform), (With<Paddle>,  Without<Ball>)>,
    mut collision_events: EventWriter<CollisionEvent>
) {
    let (mut transform) = transforms.single_mut();
    transform.translation = transform.translation + game.speed * time.delta_seconds();
    if (transform.translation.x <= CLAMP_MIN_BALL_X || transform.translation.x >= CLAMP_MAX_BALL_X)
    {
        game_state.set(GameState::End);
    }
    if (transform.translation.y <= CLAMP_MIN_BALL_Y) {
        game.speed.y = -game.speed.y;
    } else if (transform.translation.y >= CLAMP_MAX_BALL_Y) {
        game.speed.y = -game.speed.y;
    }
    transform.translation = Vec3::clamp(
        transform.translation,
        Vec3::new(CLAMP_MIN_BALL_X, CLAMP_MIN_BALL_Y, 0.0),
        Vec3::new(CLAMP_MAX_BALL_X, CLAMP_MAX_BALL_Y, 0.0),
    );

    for (paddle_transform) in &paddle_query {
        let collision = collide(
            transform.translation,
            transform.scale.truncate(),
            paddle_transform.translation,
            paddle_transform.scale.truncate(),
        );
        if let Some(collision) = collision {
            // Sends a collision event so that other systems can react to the collision
            collision_events.send_default();
            // reflect the ball when it collides
            let mut reflect_x = false;
            let mut reflect_y = false;

            // only reflect if the ball's velocity is going in the opposite direction of the
            // collision
            match collision {
                Collision::Left => reflect_x = game.speed.x > 0.0,
                Collision::Right => reflect_x = game.speed.x < 0.0,
                Collision::Top => reflect_y = game.speed.y < 0.0,
                Collision::Bottom => reflect_y = game.speed.y > 0.0,
                Collision::Inside => { /* do nothing */ }
            }

            // reflect velocity on the x-axis if we hit something on the x-axis
            if reflect_x {
                game.speed.x = -game.speed.x;
            }

            // reflect velocity on the y-axis if we hit something on the y-axis
            if reflect_y {
                game.speed.y = -game.speed.y;
            }
        }
    }
}
fn mouse_click_system(
    time: Res<Time>,
    mut game_state: ResMut<NextState<GameState>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut transforms: Query<(&mut Transform), With<Paddle>>,
) {
    for mut transform in &mut transforms {
        if mouse_button_input.pressed(MouseButton::Left) {
            transform.translation.y = transform.translation.y - 100.0 * time.delta_seconds();

            //println!("bird click: {:?}",f32::to_radians(60.0));
        } else if mouse_button_input.pressed(MouseButton::Right) {
            transform.translation.y = transform.translation.y + 100.0 * time.delta_seconds();
        }
        transform.translation.y = transform
            .translation
            .y
            .clamp(CLAMP_MIN_PADDLE_Y, CLAMP_MAX_PADDLE_Y);
    }
}

fn update_change_state(
    mut game_state: ResMut<NextState<GameState>>,
    mut timer: ResMut<SplashTimer>,
    time: Res<Time>,
) {
    if timer.tick(time.delta()).finished() {
        game_state.set(GameState::Game);
    }
}
