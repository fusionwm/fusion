use anyhow::Context;
use cargo_manifest::{Manifest, MaybeInherited};
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
    output_file: PathBuf,
    is_release: bool,
}

impl BuildEnvironment {
    fn new(root: &Path, output: Option<String>, is_release: bool) -> anyhow::Result<Self> {
        let manifest = Manifest::from_path("Cargo.toml")?;
        let package = manifest
            .package
            .ok_or_else(|| anyhow::anyhow!("This is not a Cargo package"))?;

        let package_name = &package.name;
        let plugin_name = {
            let package_version = match package.version() {
                MaybeInherited::Inherited { workspace: _ } => todo!(),
                MaybeInherited::Local(version) => version,
            };
            format!("{package_name}_{package_version}")
        };

        let target = {
            let metadata = MetadataCommand::new().exec()?;
            metadata.target_directory.into_std_path_buf()
        };

        let mode = if is_release { "release" } else { "debug" };

        let manifest = root.join("manifest.toml");
        let config = root.join("config");
        let wasm = target
            .join("wasm32-wasip2")
            .join(mode)
            .join(package_name)
            .with_extension("wasm");

        let output = if let Some(output) = output {
            if output.starts_with("..") {
                PathBuf::new().join(root).join(output)
            } else if output.starts_with('.') {
                root.into()
            } else {
                output.into()
            }
        } else {
            target.join("plugins")
        };

        let output_file = output.join(&plugin_name).with_extension("fsp");

        Ok(Self {
            manifest,
            config,
            wasm,
            output,
            output_file,
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

            let manifest = read_manifest(&build.manifest)?;

            println!("Building WASM module...");
            cargo_build(build.cargo_flags())?;

            let wasm = read_wasm(&build.wasm)?;

            let Some(output_dir) = build.output.parent() else {
                anyhow::bail!("Invalid output path");
            };
            std::fs::create_dir_all(output_dir)?;

            create_zip(&manifest, &wasm, &build.config, &build.output_file)?;

            println!(
                "Done! Output: {}",
                build
                    .output_file
                    .canonicalize()
                    .with_context(|| format!(
                        "Cannot canonicalize output path: {}",
                        build.output_file.display()
                    ))?
                    .display()
            );
        }
    }

    Ok(())
}

fn create_zip(
    manifest: &[u8],
    wasm: &[u8],
    config_path: &Path,
    output_file: &Path,
) -> anyhow::Result<()> {
    let mut writer = ZipWriter::new(std::fs::File::create(output_file)?);
    writer.start_file("manifest.toml", FileOptions::DEFAULT)?;
    writer.write_all(manifest)?;

    writer.start_file("module.wasm", FileOptions::DEFAULT)?;
    writer.write_all(wasm)?;

    write_config(&mut writer, config_path)
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
    let status = std::process::Command::new("cargo")
        .arg("build")
        .arg("--target")
        .arg("wasm32-wasip2")
        .args(args)
        .status()?;

    if !status.success() {
        anyhow::bail!("Cargo build failed with status: {status}");
    }

    Ok(())
}
