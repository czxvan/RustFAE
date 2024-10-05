use std::process::{Command, exit};
use std::path::{Path, PathBuf};
use std::fs;
use crate::ImageType;

pub fn generate_image(rootfs: &str, image: &str, image_type: &ImageType) {
    let image_path = get_unique_file_name(image);
    let image = image_path.to_str().unwrap();

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
            let output = Command::new("bash")
                .arg("-c")
                .arg(format!("echo -e 'g\\nn\\n\\n\\n\\nw' | sudo fdisk {}", &nbd_device))
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
            let output = Command::new("sudo")
                .args(&["mkfs.ext4", &format!("{}p1", &nbd_device)])
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
    copy_dir_recursive(rootfs, mount_point)


}

fn copy_dir_recursive(src: &str, dst: &str) {
    let mut src_path = Path::new(src).to_path_buf();
    src_path.push("*");
    let wild_src = src_path.as_os_str().to_str().unwrap();

    let output = Command::new("sudo")
                .args(&["bash", "-c"])
                .arg(&format!("cp -r {} {}", wild_src, dst))
                .output();

    match output {
        Ok(output) if output.status.success() => {
            println!("Successfully copy dir from {}  to: {}", src, dst);
        }
        Ok(output) => {
            let error_message = String::from_utf8_lossy(&output.stderr);
            eprintln!("Failed copy dir: {}", error_message);
            exit(1);
        }
        Err(err) => {
            eprintln!("Command execution failed: {}", err);
            exit(1);
        }
    }
}

fn find_first_unused_nbd() -> Option<String>{
    // List all NBD devices
    let all_nbd_devices = get_all_nbds().unwrap();
    let active_nbd_devices = get_active_nbds().unwrap();
    // Identify and return the first unused NBD device
    for device in all_nbd_devices {
        if !active_nbd_devices.contains(&device) {
            return Some(device);
        }
    }
    None
}

pub fn get_all_nbds() -> Option<Vec<String>> {
    let all_nbd_output = Command::new("sh")
        .arg("-c")
        .arg("ls /dev/nbd* | grep -o '/dev/nbd[0-9]\\+' | uniq")
        .output()
        .ok()?;

    let mut all_nbd_devices: Vec<String> = String::from_utf8(all_nbd_output.stdout).ok()?
        .lines()
        .map(|s| s.trim().to_string())
        .collect();

    all_nbd_devices.sort_by(|a, b| {
            let a_num = a.trim_start_matches("/dev/nbd").parse::<usize>().unwrap_or(usize::MAX);
            let b_num = b.trim_start_matches("/dev/nbd").parse::<usize>().unwrap_or(usize::MAX);
            a_num.cmp(&b_num)
        });
    
    Some(all_nbd_devices)
}

pub fn get_active_nbds() -> Option<Vec<String>> {
    // List currently active NBD devices
    let active_nbd_output = Command::new("sh")
        .arg("-c")
        .arg("ps ax | grep -o 'nbd[0-9]\\+' | uniq")
        .output()
        .ok()?;

    let active_nbd_devices: Vec<String> = String::from_utf8(active_nbd_output.stdout).ok()?
        .lines()
        .filter_map(|s| {
            let name = s.trim();
            if name.starts_with("nbd") {
                Some(format!("/dev/{}", name))
            } else {
                None
            }
        })
        .collect();

    Some(active_nbd_devices)
}

#[allow(dead_code)]
pub fn get_unused_nbds() -> Option<Vec<String>> {
    let all_nbd_devices = get_all_nbds().unwrap();
    let active_nbd_devices = get_active_nbds().unwrap();
    
    let unused_ndb_devices = all_nbd_devices
        .iter()
        .filter(|s| !active_nbd_devices.contains(s))
        .cloned()
        .collect();

    Some(unused_ndb_devices)
}

pub fn get_unique_file_name(image: &str) -> PathBuf  {
    // Get the extension of the image
    let original_path = Path::new(image);

    let directory = original_path.parent().unwrap_or_else(|| Path::new(""));
    let extension = original_path.extension().unwrap_or_default().to_str().unwrap_or("");

    // Create a new base name without the extension
    let base_name = original_path
        .file_stem()
        .unwrap_or_else(|| original_path.as_os_str())
        .to_str()
        .unwrap_or("");

    // Check if the image already exists and create a new name if necessary
    let mut image_path = original_path.to_path_buf();
    let mut count = 1;

    while image_path.exists() {
        // Create a new image name with incrementing count
        let new_image_name = format!("{}-{}.{extension}", base_name, count);
        image_path = directory.join(new_image_name);
        count += 1;
    }

    image_path
}