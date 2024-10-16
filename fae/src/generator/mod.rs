use std::process::{Command, exit};
use std::path::Path;
use std::fs;

pub mod utils;
mod image;
use utils::*;
use crate::utils::Arch;
use crate::ImageType;
use image::{fix_image, enhance_image};


pub fn generate_image(rootfs: &str, image: &str, image_type: &ImageType, arch: &Arch) {
    // let image_path = get_unique_file_name(image);
    // let image = image_path.to_str().unwrap();

    // load nbd module
    if Command::new("sudo").args(&["modprobe", "nbd"]).output().is_err() {
        eprintln!("modprobe is not installed. Please install it and try again.");
        exit(1);
    }

    let nbd_device = find_first_unused_nbd()
                .expect("Failed to find any unused nbd device!");

    println!("image: {}", image);
    println!("nbd_device: {}", nbd_device);

    // Check if binwalk is installed
    if Command::new("qemu-img").arg("-h").output().is_err() {
        eprintln!("qemu-img is not installed. Please install it and try again.");
        exit(1);
    }

    // get image type
    let image_type_str = match image_type {
        ImageType::Qcow2 => "qcow2",
        ImageType::Raw => "raw",
    };

    // create image with qemu-img
    let output =  Command::new("qemu-img")
            .arg("create")
            .arg("-f")
            .arg(image_type_str)
            .arg(image)
            .arg("1G")
            .output();
    match output {
        Ok(output) if output.status.success() => {
            println!("Successfully created image: {}", image);
        }
        Ok(output) => {
            let error_message = String::from_utf8_lossy(&output.stderr);
            eprintln!("Failed to create image: {}", error_message);
            exit(1);
        }
        Err(err) => {
            eprintln!("Command execution failed: {}", err);
            exit(1);
        }
    };

    // create mount point
    let image_path = Path::new(image);
    let mount_point_path = image_path.parent().unwrap().join("temp_image");
    let mount_point = mount_point_path.as_os_str().to_str().unwrap();
    if let Err(err) = fs::create_dir_all(&mount_point) {
        eprintln!("create mount point failed: {}", err);
        exit(1);
    } else {
        println!("mount_point: {}", mount_point);
    }

    match image_type {
        ImageType::Qcow2 => {
            // mount the qcow2 image
            let output = Command::new("sudo")
                                    .args(&["modprobe", "nbd"]).output();
            match output {
                Ok(output) if output.status.success() => {
                    println!("Successfully loaded nbd module.");
                }
                Ok(output) => {
                    let error_message = String::from_utf8_lossy(&output.stderr);
                    eprintln!("Failed to load nbd module: {}", error_message);
                    exit(1);
                }
                Err(err) => {
                    eprintln!("Command execution failed: {}", err);
                    exit(1);
                }
            }

            // Connect image with an nbd device
            let output = Command::new("sudo")
                .args(&["qemu-nbd", "-c", &nbd_device, image])
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    println!("Successfully connect image with nbd device: {}", &nbd_device)
                }
                Ok(output) => {
                    let error_message = String::from_utf8_lossy(&output.stderr);
                    eprintln!("Failed to connect image with nbd device: {}", error_message);
                    exit(1);
                }
                Err(err) => {
                    eprintln!("Command execution failed: {}", err);
                    exit(1);
                }
            }

            // Create partition for nbd device, must be bash not sh
            // must use dos rather than gpt partition
            let output = Command::new("bash")
                .arg("-c")
                .arg(format!("echo -e 'o\\nn\\np\\n1\\n\\n\\nw' | sudo fdisk {}", &nbd_device))
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    println!("Successfully created partition for nbd device.");
                }
                Ok(output) => {
                    let error_message = String::from_utf8_lossy(&output.stderr);
                    eprintln!("Failed to create partition for nbd device: {}", error_message);
                    exit(1);
                }
                Err(err) => {
                    eprintln!("Command execution failed: {}", err);
                    exit(1);
                }
            }
            
            // Make file system for partition 1 of the nbd device
            // better to use ext2, ext4 may fail when boot with qemu
            let output = Command::new("sudo")
                .args(&["mkfs.ext2", &format!("{}p1", &nbd_device)])
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    println!("Successfully mkfs.ext4 for {}p1", &nbd_device);
                }
                Ok(output) => {
                    let error_message = String::from_utf8_lossy(&output.stderr);
                    eprintln!("Failed to make file system for partition 1 of the nbd device: {}", error_message);
                    exit(1);
                }
                Err(err) => {
                    eprintln!("Command execution failed: {}", err);
                    exit(1);
                }
            }

            // Mount device to mount_point
            let output = Command::new("sudo")
                .args(&["mount", &format!("{}p1", &nbd_device), mount_point])
                .output();
            match output {
                Ok(output) if output.status.success() => {
                    println!("Successfully mounted device to: {}", mount_point);
                }
                Ok(output) => {
                    let error_message = String::from_utf8_lossy(&output.stderr);
                    eprintln!("Failed to mount device to {}: {}", mount_point, error_message);
                    exit(1);
                }
                Err(err) => {
                    eprintln!("Command execution failed: {}", err);
                    exit(1);
                }
            }
        }
        ImageType::Raw => {
            // mount the raw image
            let output = Command::new("sudo")
                .args(&["mount", "-o", "loop", image, mount_point])
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    println!("Successfully mounted image: {}", image);
                }
                Ok(output) => {
                    let error_message = String::from_utf8_lossy(&output.stderr);
                    eprintln!("Failed to mount image: {}", error_message);
                    exit(1);
                }
                Err(err) => {
                    eprintln!("Command execution failed: {}", err);
                    exit(1);
                }
            }
        }

    }

    // Copy files into the mounted image
    copy_dir_recursive(rootfs, mount_point);
    fix_image(mount_point);
    enhance_image(mount_point, arch);

    // umount mount_point
    umount(mount_point);

    // disconnect nbd device
    disconnect_nbd_device(&nbd_device);

}