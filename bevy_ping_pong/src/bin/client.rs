use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::{app::ScheduleRunnerPlugin, prelude::*, utils::Duration, sprite::MaterialMesh2dBundle};
use bevy_egui::EguiPlugin;
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
use bevy_ping_pong::{PlayerBundle, PingPongPlugin, PORT, PROTOCOL_ID, LocalData};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ReplicationPlugins))
        .add_plugins(PingPongPlugin)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, init_client.map(Result::unwrap))
        .add_systems(Update, bevy_ping_pong::PingPongPlugin::render_gui_client)
        .run();
}

fn init_client(
    mut commands: Commands,
    network_channels: Res<NetworkChannels>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) -> Result<(), Box<dyn Error>> {
    const ip: IpAddr = std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let server_channels_config = network_channels.get_server_configs();
    let client_channels_config = network_channels.get_client_configs();

    let client = RenetClient::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let client_id = current_time.as_millis() as u64;

    let server_addr = SocketAddr::new(ip, PORT);
    let socket = UdpSocket::bind((ip, 0))?;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };
    let transport = NetcodeClientTransport::new(current_time, authentication, socket)?;

    commands.insert_resource(client);
    commands.insert_resource(transport);

    // commands.spawn(TextBundle::from_section(
    //     format!("Client: {client_id:?}"),
    //     TextStyle {
    //         font_size: 30.0,
    //         color: Color::WHITE,
    //         ..default()
    //     },
    // ));
    let x = LocalData {
        client_id: client_id
    };

    // commands.spawn(MaterialMesh2dBundle {
    //     mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
    //     transform: Transform::default().with_scale(Vec3::splat(128.)),
    //     material: materials.add(ColorMaterial::from(Color::PURPLE)),
    //     ..default()
    // });
    commands.add(|world: &mut World| {
        world.insert_resource(x)
    });
    Ok(())
}
