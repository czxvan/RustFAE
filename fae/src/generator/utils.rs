use std::path::{Path, PathBuf};
use std::process::Command;

/// For file
pub fn copy_dir_recursive(src: &str, dst: &str) {
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
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("Command execution failed: {}", err);
            std::process::exit(1);
        }
    }
}

#[allow(dead_code)]
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


/// For nbd device
pub fn find_first_unused_nbd() -> Option<String>{
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
        .arg("ps ax | grep -o '/dev/nbd[0-9]\\+' | uniq")
        .output()
        .ok()?;

    let active_nbd_devices: Vec<String> = String::from_utf8(active_nbd_output.stdout).ok()?
        .lines()
        .filter_map(|s| {
            let name = s.trim();
            if name.starts_with("nbd") {
                Some(format!("{}", name))
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

pub fn umount(mount_point: &str) {
    let output = Command::new("sudo")
            .args(&["umount", mount_point])
            .output()
            .expect("Failed to execute command: umount");

    if !output.status.success() {
        eprintln!("Failed to umount: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    } else {
        println!("umount: {}", mount_point);
    }
}

pub fn mkdir_p(directory: &str) {
    let output = Command::new("sudo")
            .args(&["mkdir", "-p", directory])
            .output()
            .expect("Failed to execute command: mkdir");

    if !output.status.success() {
        eprintln!("Failed to mkdir: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    } else {
        println!("mkdir: {}", directory);
    }
}

#[derive(Debug)]
pub struct Device {
    name: String,
    device_type: String, // c or b
    mode: u32,  // 权限模式
    rdev: (u32, u32), // 主设备号和次设备号
}

impl Device {
    pub fn new(name: &str, device_type: &str, mode: u32, rdev: (u32, u32)) -> Self {
        Device {
            name: name.to_string(),
            device_type: device_type.to_string(),
            mode,
            rdev,
        }
    }

    pub fn create(&self) {
        // 使用 mknod 创建设备节点
        let output = Command::new("sudo")
            .arg("mknod")
            .arg("-m").arg(self.mode.to_string())
            .arg(&self.name)
            .arg(&self.device_type)
            .arg(self.rdev.0.to_string())
            .arg(self.rdev.1.to_string())
            .output()
            .expect("Failed to create device node");

        if !output.status.success() {
            eprintln!("mknod {}: {}", self.name, String::from_utf8_lossy(&output.stderr));
        } else {
            println!("mknod: {}", self.name);
        }
    }
}

pub fn disconnect_nbd_device(nbd_device: &str) {
    let output = Command::new("sudo")
                .args(&["qemu-nbd", "-d", nbd_device])
                .output()
                .expect("Failed to execute command: qemu-nbd");

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        eprintln!("Failed to disconnect {}: {}", nbd_device, error_message);
        std::process::exit(1);
    } else {
        println!("disconnect: {}", nbd_device);
    }
}

/// clean functions
pub fn umount_temp_images() {
    let output = Command::new("bash")
        .arg("-c")
        .arg("mount | grep temp_image | grep -o '[^ ]*temp_image'")
        .output()
        .expect("Failed to execute command: mount");

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        println!("There is no mounted temp images {}", error_message);
    } else {
        println!("get mounted temp images: {}", String::from_utf8_lossy(&output.stdout));
    }

    let mounted_temp_images: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .collect();
    for temp_image in mounted_temp_images {
        umount(&temp_image);
    };

}

pub fn disconnect_nbd_divices() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("ps ax | grep -o '/dev/nbd[0-9]\\+'")
        .output()
        .expect("Failed to execute command: ps ax");

    if !output.status.success() {
        println!("There is no active nbd devices {}", String::from_utf8_lossy(&output.stderr));
    } else {
        println!("get active nbd devices: {}", String::from_utf8_lossy(&output.stdout));
    }

    let active_nbd_devices: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .collect();
    for nbd_device in active_nbd_devices {
        disconnect_nbd_device(&nbd_device)
    };
}