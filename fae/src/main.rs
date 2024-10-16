use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
mod extractor;
mod generator;
mod emulator;
mod utils;
use extractor::extract_firmware;
use generator::generate_image;
use emulator::run_emulation;
use utils::*;

#[derive(Parser)]
#[command(name = "firmware_tool")]
#[command(version = "1.0")]
#[command(author = "你的名字")]
#[command(about = "用于提取和模拟固件的工具")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand, Deserialize, Serialize,)]
enum Command {
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
        input: String,
    },
    /// clean
    Clean {},
    /// Umount
    Umount,
    /// run tasks according to task file
    RunTasks {
        /// 
        task_file: String,
    },

}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Command::Extract { firmware, directory } => {
            extract_firmware(firmware, directory);
        }
        Command::Generate { rootfs, image, type_image, arch} => {
            generate_image(rootfs, image, type_image, arch);
        }
        Command::Emulate { image, arch} => {
            run_emulation(image, arch);
        }
        Command::GenerateAndEmulate {rootfs, image, type_image, arch} => {
            generate_image(rootfs, image, type_image, arch);
            run_emulation(image, arch);
        }
        Command::RunTasks { task_file } => {
            println!("Run task: {}", task_file);
            run_tasks(task_file);

        }
        Command::Test { input } => {
            println!("{}", test_func(&input));
        }
        Command::Clean {  } => {
            utils::umount_temp_images();
            utils::disconnect_nbd_divices();
        }
        Command::Umount {  } => {
            utils::umount_temp_images();
        }
    }
}


// use std::path::Path;
fn test_func(_path: &str) -> String {
    let tasks: Tasks = Tasks {
        extract: Some(Extract { firmware: "czx".to_string() , directory: "czx".to_string() }),
        generate: None,
        emulate: Some(Emulate { image: "czx".to_string(), arch: Arch::Arm }),    
    };

    let toml_string = toml::to_string(&tasks).expect("Failed to serialize config");
    toml_string
}

fn run_tasks(task_file: &str) {
    let config_content  = std::fs::read_to_string(task_file).expect("Failed to read task file");
    let tasks: Tasks = toml::de::from_str(&config_content).expect("Unable to parse TOML");
    
    if let Some(Extract {firmware, directory}) = &tasks.extract {
        println!("Extracting firmware {} to directory {}", firmware, directory);
        extract_firmware(firmware, directory);
    }
    if let Some(Generate {rootfs, image,type_image, arch}) = &tasks.generate {
        println!("Generating firmware {} for architecture {:?}", image, arch);
        generate_image(rootfs, image, type_image, arch);
    }
    if let Some(Emulate {image, arch}) = &tasks.emulate {
        println!("Emulating firmware {} on architecture {:?}", image, arch);
        run_emulation(image, arch);
    }

}