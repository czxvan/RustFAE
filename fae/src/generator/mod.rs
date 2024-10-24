use std::process::{Command, exit};

pub mod utils;
mod image;
use utils::*;
use crate::utils::Arch;
use crate::ImageType;
use image::*;


pub fn generate_image(rootfs: &str, image: &str, image_type: &ImageType, arch: &Arch) {
    // let image_path = get_unique_file_name(image);
    // let image = image_path.to_str().unwrap();

    // load nbd module
    if Command::new("sudo").args(&["modprobe", "nbd"]).output().is_err() {
        eprintln!("modprobe is not installed. Please install it and try again.");
        exit(1);
    }

    println!("image: {}", image);
    

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
    create_image(image_type_str, image);

    // create mount point
    let mount_point_string = create_mount_point(image);
    let mount_point = mount_point_string.as_str();

    let immediate_device = match image_type {
        ImageType::Qcow2 => {
            // mount the qcow2 image
            mount_qcow2_image(image, &mount_point)
        }
        ImageType::Raw => {
            // [TODO] mount the raw image
            mount_raw_image(image, mount_point)
        }
    };

    // Copy files into the mounted image
    copy_dir_recursive(rootfs, mount_point);
    fix_image(mount_point);
    enhance_image(mount_point, arch);

    // umount mount_point
    umount(mount_point);

    match image_type {
        ImageType::Qcow2 => {
            // disconnect nbd device
            disconnect_nbd_device(&immediate_device);
        }
        ImageType::Raw => {
            // [TODO] disconnect raw device
        }
        
    }

}