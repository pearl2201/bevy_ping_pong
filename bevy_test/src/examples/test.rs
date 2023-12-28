// use bevy::{app::ScheduleRunnerPlugin, prelude::*, utils::Duration};
// use bevy_egui::{egui, EguiContexts, EguiPlugin};
// use splash::SplashPlugin;
// mod splash;
// mod helper;
// use helper::despawn_screen;
// #[derive(Component)]
// struct Person;

// #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
// enum GameState {
//     #[default]
//     Splash,
//     Menu,
//     Game,
// }

// #[derive(Component)]
// struct Name(String);

// fn add_people(mut commands: Commands) {
//     commands.spawn((Person, Name("Elaina Proctor".to_string())));
//     commands.spawn((Person, Name("Renzo Hume".to_string())));
//     commands.spawn((Person, Name("Zayna Nieves".to_string())));
// }



// fn main() {
//     // This app runs once
//     // App::new()
//     //     .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_once()))
//     //     .add_systems(Update, hello_world)
//     //     .run();

//     // App::new()
//     //     .add_plugins(
//     //         MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
//     //             1.0 / 60.0,
//     //         ))),
//     //     )
//     //     .insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
//     //     .add_systems(Startup, add_people)
//     //     .add_systems(Update, (greet_people))
//     //     .run();

//     App::new().add_plugins(DefaultPlugins)
//     .add_plugins(EguiPlugin)
//     .add_plugins(SplashPlugin)
//     .add_systems(Update, ui_example_system).run();
// }

// fn hello_world() {
//     println!("hello world!");
// }

// fn ui_example_system(mut contexts: EguiContexts) {
//     egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
//         ui.label("world");
//     });
// }
// fn counter(mut state: Local<CounterState>) {
//     if state.count % 60 == 0 {
//         println!("{}", state.count);
//     }
//     state.count += 1;
// }

// #[derive(Default)]
// struct CounterState {
//     count: u32,
// }

// #[derive(Resource)]
// struct GreetTimer(Timer);

// fn greet_people(
//     time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
//     // update our timer with the time elapsed since the last update
//     // if that caused the timer to finish, we say hello to everyone
//     if timer.0.tick(time.delta()).just_finished() {
//         for name in &query {
//             println!("hello {}!", name.0);
//         }
//     }
// }

