use std::process::{Command, Stdio};
use crate::utils::Arch;

pub mod utils;
use utils::*;

pub fn run_emulation(image: &str, arch: &Arch, debug: &bool) {
    init_network();
    let kernel;
    let qemu;
    let machine;
    let drive_if;
    let root_device;
    let net_device;
    match arch {
        Arch::Arm => {
            qemu = "qemu-system-arm";
            kernel = "../binaries/kernel/virgin/zImage.arm.4.virgin";
            machine = "virt-2.10";
            root_device = "/dev/vda1";
            drive_if = "none";
            net_device = "virtio-net-device";
        }
        Arch::Mips => {
            qemu = "qemu-system-mips";
            kernel = "../binaries/kernel/virgin/vmlinux.mips.4.virgin"; // 3.2.0.malta
            machine = "malta";
            root_device = "/dev/sda1";
            drive_if = "ide";
            net_device = "pcnet";
        }
        Arch::Mipsel => {
            qemu = "qemu-system-mipsel";
            kernel = "../binaries/kernel/vmlinux.mipsel.4";
            machine = "";
            root_device = "";
            drive_if = "";
            net_device = "e1000";
        }
    }

    let mut sudo = Command::new("sudo");
    let process  = sudo
        .args(&[
            qemu,
            "-kernel", kernel,
            "-M", machine,
            "-drive", &format!("if={drive_if},format=qcow2,file={image},id=rootfs"), // "-hda", image,
            "-m", "256M",
            "-nographic",
            "-append", &format!("root={root_device} rw init=preInit.sh"), // sda1 /dev/mmcblk0
            "-netdev", "tap,id=net0,ifname=tap-qemu,script=no,downscript=no",
            "-device", &format!("{net_device},netdev=net0,id=nic1")
        ]);
    if *debug {
        process.args(&["-s", "-S"]);
    }
    match arch {
        Arch::Arm => {
            process.args(&["-device", "virtio-blk-device,drive=rootfs"]);
        }
        _ => {}
    }

    let mut process = process.stdin(Stdio::inherit()) // 允许向 QEMU 发送输入
        .stdout(Stdio::inherit()) // 捕获 QEMU 的输出
        .stderr(Stdio::inherit()) // 捕获 QEMU 的错误输出
        .spawn()
        .expect(&format!("Failed to execute command: {}", qemu));
    let _ = process.wait().expect("QEMU process wasn't running");
}
