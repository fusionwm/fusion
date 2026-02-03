use cargo_manifest::Manifest;
use cargo_metadata::MetadataCommand;
use clap::{Parser, Subcommand};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;
use zip::{ZipWriter, write::FileOptions};

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Build {
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        release: bool,

        #[arg(short, long)]
        output: Option<String>,
    },
}

struct BuildEnvironment {
    manifest: PathBuf,
    config: PathBuf,
    wasm: PathBuf,
    output: PathBuf,
    is_release: bool,
}

impl BuildEnvironment {
    fn new(root: &Path, output: Option<String>, is_release: bool) -> anyhow::Result<Self> {
        let manifest = Manifest::from_path("Cargo.toml")?;
        let package = manifest
            .package
            .ok_or_else(|| anyhow::anyhow!("This is not a Cargo package"))?;

        let target = if let Some(output) = output {
            if output.starts_with("..") {
                PathBuf::new().join(root).join(output)
            } else if output.starts_with('.') {
                root.into()
            } else {
                output.into()
            }
        } else {
            let metadata = MetadataCommand::new().exec()?;
            metadata.target_directory.into_std_path_buf()
        };

        let mode = if is_release { "release" } else { "debug" };

        let manifest = root.join("manifest.toml");
        let config = root.join("config");
        let wasm = target
            .join("wasm32-wasip2")
            .join(mode)
            .join(&package.name)
            .with_extension("wasm");

        let output = target.join(&package.name).with_extension("fus");

        Ok(Self {
            manifest,
            config,
            wasm,
            output,
            is_release,
        })
    }

    fn cargo_flags(&self) -> Option<&str> {
        if self.is_release {
            Some("--release")
        } else {
            None
        }
    }
}

fn main() -> anyhow::Result<()> {
    let plugin_root = std::env::current_dir()?;

    let cli = Cli::parse();
    let Some(command) = cli.command else {
        return Ok(());
    };

    match command {
        Commands::Build { release, output } => {
            let build = BuildEnvironment::new(&plugin_root, output, release)?;

            // 4. Проверки и копирование манифеста
            let manifest = read_manifest(&build.manifest)?;

            // 6. Сборка WASM
            println!("Building WASM module...");
            cargo_build(build.cargo_flags())?;

            let wasm = read_wasm(&build.wasm)?;

            println!("Done! Output: {}", build.output.display());

            let mut writer = ZipWriter::new(std::fs::File::create(build.output)?);
            writer.start_file("manifest.toml", FileOptions::DEFAULT)?;
            writer.write_all(&manifest)?;

            writer.start_file("plugin.wasm", FileOptions::DEFAULT)?;
            writer.write_all(&wasm)?;

            write_config(&mut writer, &build.config)?;
        }
    }

    Ok(())
}

fn read_manifest(manifest_path: &Path) -> anyhow::Result<Vec<u8>> {
    if !manifest_path.exists() {
        anyhow::bail!("Cannot find manifest.toml");
    }
    Ok(std::fs::read(manifest_path)?)
}

fn read_wasm(wasm_path: &Path) -> anyhow::Result<Vec<u8>> {
    if !wasm_path.exists() {
        anyhow::bail!("Cannot find built wasm at {}", wasm_path.display());
    }
    Ok(std::fs::read(wasm_path)?)
}

fn write_config(writer: &mut ZipWriter<File>, config_path: &Path) -> anyhow::Result<()> {
    writer.add_directory("config", FileOptions::DEFAULT)?;
    if !config_path.exists() {
        return Ok(());
    }

    for entry in WalkDir::new(config_path) {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(config_path)?.to_str().unwrap();

        if path.is_file() {
            writer.start_file(name, FileOptions::DEFAULT)?;
            let file_content = std::fs::read(path)?;
            writer.write_all(&file_content)?;
        } else if !name.is_empty() {
            writer.add_directory(name, FileOptions::DEFAULT)?;
        }
    }
    Ok(())
}

fn cargo_build<'a, I: IntoIterator<Item = &'a str>>(args: I) -> anyhow::Result<()> {
    std::process::Command::new("cargo")
        .arg("build")
        .arg("--target")
        .arg("wasm32-wasip2")
        .args(args)
        .spawn()?;

    Ok(())
}
