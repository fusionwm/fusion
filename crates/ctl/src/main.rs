use bincode::{Decode, Encode};
use clap::{Parser, Subcommand, ValueEnum};
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

#[derive(Debug, Clone, Encode, Decode)]
enum SocketCommandResult {
    Done,
    Modules { list: Vec<String> },
}

#[derive(Default, Debug, Copy, Clone, ValueEnum, Encode, Decode)]
enum ModuleListFilter {
    #[default]
    All,
    Failed,
    Running,
    Stopped,
}

#[derive(Subcommand, Debug, Copy, Clone, Encode, Decode)]
enum Commands {
    Modules {
        #[arg(value_enum)]
        filter: Option<ModuleListFilter>,
    },
    ReloadModule {
        #[arg(short, long)]
        id: usize,
    },
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

const SOCKET_PATH: &str = "C:\\Users\\mrapa\\nethalym-engine.sock";

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    let mut bytes = bincode::encode_to_vec(cli.command, bincode::config::standard()).unwrap();
    stream.write_all(&bytes)?;

    bytes.clear();
    stream.read_exact(&mut bytes)?;

    let (result, _): (SocketCommandResult, usize) =
        bincode::decode_from_slice(&bytes, bincode::config::standard()).unwrap();

    match result {
        SocketCommandResult::Done => {}
        SocketCommandResult::Modules { list } => {
            for module in &list {
                println!("{module}");
            }
        }
    }

    Ok(())
}
