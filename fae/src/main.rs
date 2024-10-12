use clap::{Parser, Subcommand};

mod extractor;
mod generator;
mod emulator;
mod utils;
use extractor::extract_firmware;
use generator::generate_image;
use emulator::run_emulation;
use utils::{Arch, ImageType};

#[derive(Parser)]
#[command(name = "firmware_tool")]
#[command(version = "1.0")]
#[command(author = "你的名字")]
#[command(about = "用于提取和模拟固件的工具")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// extract root filesystem from .bin firmware
    Extract {
        /// The firmware to be extracted
        #[arg(short, long)]
        firmware: String,

        /// Extract files/folders to a custom directory (default: current working directory)
        #[arg(short, long, default_value_t = String::from("../outputs"))]
        directory: String,
    },
    /// generate image according to root filesystem
    Generate {
        /// root filesystem extracted from firmware
        rootfs: String,
        /// qcow2 or raw image used to run emulation
        image: String,
        /// image type qcow2 or raw
        type_image: ImageType,
        /// arch: arm, mips, mipsel
        arch: Arch,

    },
    /// run emulation for the firmware
    Emulate {
        /// image regarded as root filesystem, qcow2 or raw image
        image: String,
        arch: Arch,
    },
    /// generate and emulate
    GenerateAndEmulate {
        /// root filesystem extracted from firmware
        rootfs: String,
        /// qcow2 or raw image used to run emulation
        image: String,
        /// image type qcow2 or raw
        type_image: ImageType,
        /// architecture: arm mips mipsel
        arch: Arch,
    },
    /// test
    Test {
        input: String
    },
    /// clean
    Clean {},
    /// Umount
    Umount

}


fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Extract { firmware, directory } => {
            extract_firmware(firmware, directory);
        }
        Commands::Generate { rootfs, image, type_image, arch} => {
            generate_image(rootfs, image, type_image, arch);
        }
        Commands::Emulate { image, arch} => {
            run_emulation(image, arch);
        }
        Commands::GenerateAndEmulate {rootfs, image, type_image, arch} => {
            generate_image(rootfs, image, type_image, arch);
            run_emulation(image, arch);
        }
        Commands::Test { input } => {
            println!("{}", test_func(&input));
        }
        Commands::Clean {  } => {
            utils::umount_temp_images();
            utils::disconnect_nbd_divices();
        }
        Commands::Umount {  } => {
            utils::umount_temp_images();
        }
    }
}

use std::path::Path;
fn test_func(_path: &str) -> String {
    let path = Path::new("../outputs/_R6300v2_V1.0.2.72_1.0.46.bin.extracted/squashfs-root/bin");
    path.canonicalize().unwrap().to_string_lossy().into_owned()
}