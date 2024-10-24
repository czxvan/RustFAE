use std::process::Command;
use std::path::{Path, PathBuf};
use crate::utils::Arch;

use super::utils::{mkdir_p, Device, find_first_unused_nbd};

fn merge_paths(base: &str, relative: &str) -> PathBuf {
    let base_path = Path::new(base);
    let relative_path = Path::new(relative);

    let merged_path = if relative_path.has_root() {
        base_path.join(relative_path.strip_prefix("/").unwrap_or(relative_path))
    } else {
        base_path.join(relative_path)
    };
    merged_path
}

fn resolve_link(path: &str) -> String {
    let input_path = Path::new(path);
    let parent_dir = input_path.parent().unwrap();

    // 尝试读取符号链接
    match std::fs::read_link(input_path) {
        Ok(target_path) => {
            return merge_paths(parent_dir.to_str().unwrap(),target_path.to_str().unwrap())
                    .to_string_lossy().into_owned();
        }
        Err(_) => {
            // 如果不是符号链接，返回输入路径
            return input_path.to_string_lossy().into_owned();
        }
    }
}

pub fn create_image(image_type_str: &str, image: &str) {
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
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("Command execution failed: {}", err);
            std::process::exit(1);
        }
    };
}

pub fn create_mount_point(image: &str) -> String {
    let image_path = Path::new(image);
    let mount_point_path = image_path.parent().unwrap().join("temp_image");
    let mount_point = mount_point_path.as_os_str().to_str().unwrap();
    if let Err(err) = std::fs::create_dir_all(&mount_point) {
        eprintln!("create mount point failed: {}", err);
        std::process::exit(1);
    } else {
        println!("mount_point: {}", mount_point);
    }
    mount_point.to_string()
}

pub fn mount_qcow2_image(image: &str, mount_point: &str) -> String {
    let output = Command::new("sudo")
                                    .args(&["modprobe", "nbd"]).output();
    match output {
        Ok(output) if output.status.success() => {
            println!("Successfully loaded nbd module.");
        }
        Ok(output) => {
            let error_message = String::from_utf8_lossy(&output.stderr);
            eprintln!("Failed to load nbd module: {}", error_message);
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("Command execution failed: {}", err);
            std::process::exit(1);
        }
    }

    let nbd_device = find_first_unused_nbd()
        .expect("Failed to find any unused nbd device!");
    println!("nbd_device: {}", &nbd_device);

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
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("Command execution failed: {}", err);
            std::process::exit(1);
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
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("Command execution failed: {}", err);
            std::process::exit(1);
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
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("Command execution failed: {}", err);
            std::process::exit(1);
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
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("Command execution failed: {}", err);
            std::process::exit(1);
        }
    }

    nbd_device
}

pub fn mount_raw_image(image: &str, mount_point: &str) -> String {
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
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("Command execution failed: {}", err);
            std::process::exit(1);
        }
    }
    "todo".to_string()
}

pub fn fix_image(mount_point: &str) {
    let dirs = [
            "/proc", "/dev/pts", "/etc_ro", "/tmp", "/var", "/run",
            "/sys", "/root", "/tmp/var", "/tmp/media", "/tmp/etc",
            "/tmp/var/run", "/tmp/home/root", "/tmp/mnt", "/tmp/opt",
            "/tmp/www", "/var/run", "/var/lock", "/usr/bin", "/usr/sbin",
            "/dev/mtd", "/dev/tts", "/dev/mtdblock"
        ];
    for dir in &dirs {
        mkdir_p(&resolve_link(merge_paths(mount_point, &dir).to_str().unwrap()));
    }
    
    let devices = [
        ("/dev/mem", "c", 660, (1, 1)),
        ("/dev/kmem", "c", 640, (1, 2)),
        ("/dev/null", "c", 666, (1, 3)),
        ("/dev/zero", "c", 666, (1, 5)),
        ("/dev/random", "c", 444, (1, 8)),
        ("/dev/urandom", "c", 444, (1, 9)),
        ("/dev/armem", "c", 666, (1, 13)),

        ("/dev/tty", "c", 666, (5, 0)),
        ("/dev/console", "c", 622, (5, 1)),
        ("/dev/ptmx", "c", 666, (5, 2)),

        ("/dev/tty0", "c", 622, (4, 0)),
        ("/dev/ttyS0", "c", 660, (4, 64)),
        ("/dev/ttyS1", "c", 660, (4, 65)),
        ("/dev/ttyS2", "c", 660, (4, 67)),
        ("/dev/ttyS3", "c", 660, (4, 66)),

        ("/dev/tts/0", "c", 660, (4, 64)),
        ("/dev/tts/1", "c", 660, (4, 65)),
        ("/dev/tts/2", "c", 660, (4, 66)),
        ("/dev/tts/3", "c", 660, (4, 67)),
    ];
    for (device_name, device_type, mode, rdev) in devices {
        let real_device_name = merge_paths(mount_point, device_name);
        Device::new(real_device_name.to_str().unwrap(), device_type, mode, rdev).create();
    }

    for i in 0..=10 {
        let real_device_name = merge_paths(mount_point, &format!("/dev/mtdblock/{}", i));
        Device::new(real_device_name.to_str().unwrap(), "b", 644, (31, i)).create();
    }

    for i in 0..=10 {
        let real_device_name = merge_paths(mount_point, &format!("/dev/mtdblock{}", i));
        Device::new(real_device_name.to_str().unwrap(), "b", 644, (31, i)).create();
    }

    // [TODO] 创建设备节点 /dev/null /dev/mtd* /dev/mtdblock* /dev/gpio
}

pub fn enhance_image(mount_point: &str, arch: &Arch) {
    let arch_str= arch.to_str();
    let binaries_base = "../binaries";
    let binaries = [
        (format!("{}/agent/agent.{}", binaries_base, arch_str), "agent"),
        (format!("{}/busybox/busybox.{}", binaries_base, arch_str), "busybox"),
        (format!("{}/preInit/preInit.sh", binaries_base), "preInit.sh"),
    ];

    for (binary, dest) in &binaries {
        let output = Command::new("sudo")
            .args(&["cp", binary, merge_paths(mount_point, dest).to_str().unwrap()])
            .output()
            .expect("Failed to execute command: cp");

        if !output.status.success() {
            println!("cp error: {}", String::from_utf8_lossy(&output.stderr));
            std::process::exit(1);
        }
    }
    
}