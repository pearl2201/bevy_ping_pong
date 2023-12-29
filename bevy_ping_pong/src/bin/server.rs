use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::{
    a11y::AccessibilityPlugin,
    app::ScheduleRunnerPlugin,
    core_pipeline::CorePipelinePlugin,
    diagnostic::DiagnosticsPlugin,
    input::InputPlugin,
    prelude::*,
    render::{settings::WgpuSettings, RenderPlugin},
    scene::ScenePlugin,
    time::TimePlugin,
    utils::Duration,
    winit::WinitPlugin,
};
use bevy_ping_pong::{PingPongPlugin, PlayerBundle, PORT, PROTOCOL_ID};
use bevy_replicon::replicon_core::NetworkChannels;
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

fn main() {
    App::new()
        .add_plugins((
            // //DefaultPlugins
            // DefaultPlugins.set(Womd,RenderPlugin {
            //     render_creation: WgpuSettings {
            //         backends: None,
            //         ..default()
            //     }
            //     .into(),
            // }).set(TimePlugin {

            // }),
            MinimalPlugins,
            // WindowPlugin {
            //     primary_window: None,
            //     exit_condition: bevy::window::ExitCondition::DontExit,
            //     ..Default::default()
            // }
        ))
        .add_plugins((ReplicationPlugins, PingPongPlugin))
        .add_systems(Startup, init_server.map(Result::unwrap))
        .add_systems(Startup, bevy_ping_pong::PingPongPlugin::init_system_server)
        .run();
}

fn init_server(
    mut commands: Commands,
    network_channels: Res<NetworkChannels>,
) -> Result<(), Box<dyn Error>> {
    let server_channels_config = network_channels.get_server_configs();
    let client_channels_config = network_channels.get_client_configs();

    let server = RenetServer::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let public_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), PORT);
    let socket = UdpSocket::bind(public_addr)?;
    let server_config = ServerConfig {
        current_time,
        max_clients: 2,
        protocol_id: PROTOCOL_ID,
        authentication: ServerAuthentication::Unsecure,
        public_addresses: vec![public_addr],
    };
    let transport = NetcodeServerTransport::new(server_config, socket)?;

    commands.insert_resource(server);
    commands.insert_resource(transport);

    commands.spawn(TextBundle::from_section(
        "Server",
        TextStyle {
            font_size: 30.0,
            color: Color::WHITE,
            ..default()
        },
    ));
    //commands.spawn(PlayerBundle::new(SERVER_ID, Vec2::ZERO, Color::GREEN));
    println!("init system");
    Ok(())
}
