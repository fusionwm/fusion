use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

use clap::{Parser, Subcommand};
use comfy_table::{
    Cell, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL,
};
use fusion_socket_protocol::{
    CompositorRequest, FUSION_CTL_SOCKET_DEFAULT, GetPluginListRequest, GetPluginListResponse,
    PingRequest, PingResponse, Plugin, RestartPluginRequest, RestartPluginResponse,
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Ping,
    #[command(subcommand)]
    Plugins(PluginCommands),
}

#[derive(Subcommand, Clone, Debug)]
#[clap(rename_all = "snake_case")]
enum PluginCommands {
    List,
    Restart { plugin_id: String },
}

fn print_plugin_table(plugins: &[Plugin]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Name", "Id", "Status", "Version"]);

    for plugin in plugins {
        table.add_row(vec![
            Cell::new(&plugin.name),
            Cell::new(&plugin.id),
            Cell::new(&plugin.status),
            Cell::new(&plugin.version),
        ]);
    }

    println!("{table}");
}

fn send_request(
    socket: &mut UnixStream,
    request: impl Into<CompositorRequest>,
) -> anyhow::Result<()> {
    let request = request.into();
    let buf = postcard::to_stdvec_cobs(&request).unwrap();
    socket.write_all(&buf)?;
    Ok(())
}

fn read_request(socket: &mut UnixStream) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        socket.read_exact(&mut byte).unwrap();
        if byte[0] == 0x00 {
            break;
        }
        buf.push(byte[0]);
    }

    buf
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut socket = UnixStream::connect(FUSION_CTL_SOCKET_DEFAULT)?;
    match cli.command {
        Commands::Ping => {
            send_request(&mut socket, PingRequest)?;
            let mut bytes = read_request(&mut socket);

            postcard::from_bytes_cobs::<PingResponse>(&mut bytes)?;
            println!("Pong!");
        }
        Commands::Plugins(command) => match command {
            PluginCommands::List => {
                send_request(&mut socket, GetPluginListRequest)?;
                let mut bytes = read_request(&mut socket);

                match postcard::from_bytes_cobs::<GetPluginListResponse>(&mut bytes)? {
                    GetPluginListResponse::Ok(plugins) => print_plugin_table(&plugins),
                    GetPluginListResponse::Error(error) => println!("Error: {error}"),
                }
            }
            PluginCommands::Restart { plugin_id } => {
                send_request(&mut socket, RestartPluginRequest { plugin_id })?;
                let mut bytes = read_request(&mut socket);
                match postcard::from_bytes_cobs::<RestartPluginResponse>(&mut bytes)? {
                    RestartPluginResponse::Ok => println!("Ok"),
                    RestartPluginResponse::Error(error) => println!("Error: {error}"),
                }
            }
        },
    }

    Ok(())
}
