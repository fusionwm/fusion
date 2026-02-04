use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

use clap::{Parser, Subcommand};
use comfy_table::{
    Cell, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL,
};
use fusion_socket_protocol::{
    CompositorRequest, CompositorResponse, FUSION_CTL_SOCKET_VAR, Plugin,
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    List(ListCommand),
    Restart {
        plugin_id: String,
    },
}

#[derive(Subcommand, Clone, Debug)]
#[clap(rename_all = "snake_case")]
enum ListCommand {
    Plugins,
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

    // Set the default alignment for the third column to right
    //let column = table.column_mut(2).expect("Our table has three columns");
    //column.set_cell_alignment(CellAlignment::Right);

    println!("{table}");
}

fn send_request(socket: &mut UnixStream, request: &CompositorRequest) -> anyhow::Result<()> {
    let buf = postcard::to_stdvec_cobs(request).unwrap();
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
    //let path = std::env::var(FUSION_CTL_SOCKET_VAR)
    //    .expect("FUSION_CTL_SOCKET environment variable is not set");

    let path = fusion_socket_protocol::FUSION_CTL_SOCKET_DEFAULT;

    let cli = Cli::parse();

    let mut socket = UnixStream::connect(path)?;
    match cli.command {
        Commands::List(command) => match command {
            ListCommand::Plugins => {
                send_request(&mut socket, &CompositorRequest::GetPlugins)?;
                let mut bytes = read_request(&mut socket);

                match postcard::from_bytes_cobs::<CompositorResponse>(&mut bytes)? {
                    CompositorResponse::Plugins(plugins) => print_plugin_table(&plugins),
                    CompositorResponse::Error(_) => println!("Error"),
                    CompositorResponse::Ok => println!("OK"),
                }
            }
        },
        Commands::Restart { plugin_id } => {
            send_request(&mut socket, &CompositorRequest::Restart { plugin_id })?;
            let mut bytes = read_request(&mut socket);
            match postcard::from_bytes_cobs::<CompositorResponse>(&mut bytes)? {
                CompositorResponse::Plugins(plugins) => unreachable!(),
                CompositorResponse::Error(_) => println!("Error"),
                CompositorResponse::Ok => println!("OK"),
            }
        }
    }

    Ok(())
}
