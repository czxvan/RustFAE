use std::process::Command;
use std::path::{Path, PathBuf};
use crate::utils::Arch;

use super::utils::{mkdir_p, Device};

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