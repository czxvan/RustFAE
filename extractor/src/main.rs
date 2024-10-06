use clap::{Parser, Subcommand};

mod extractor;
mod generator;
mod emulator;
mod utils;
use extractor::extract_firmware;
use generator::generate_image;
use emulator::run_emulation;


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
        #[arg(short, long)]
        rootfs: String,
        /// qcow2 or raw image used to run emulation
        #[arg(short, long)]
        image: String,
    },
    /// run emulation for the firmware
    Emulate {
        /// image regarded as root filesystem, qcow2 or raw image
        image: String,
    },
    /// test
    Test {},
    /// clean
    Clean {},
    /// Umount
    Umount

}



enum ImageType {
    Qcow2,
    #[allow(dead_code)]
    Raw
}


fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Extract { firmware, directory } => {
            extract_firmware(firmware, directory);
        }
        Commands::Generate { rootfs, image} => {
            generate_image(rootfs, image, &ImageType::Qcow2);
        }
        Commands::Emulate { image } => {
            run_emulation(image);
        }
        Commands::Test {  } => {
            test_func();
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

use std::env;
fn test_func () {
    // 获取当前工作目录
    let current_dir = env::current_dir().expect("Failed to get current directory");
    println!("Current directory: {:?}", current_dir);
}