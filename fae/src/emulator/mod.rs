use std::process::{Command, Stdio};
use crate::utils::Arch;

pub mod utils;
use utils::*;

pub fn run_emulation(image: &str, arch: &Arch) {
    init_network();
    let kernel;
    let qemu;
    let machine;
    match arch {
        Arch::Arm => {
            qemu = "qemu-system-arm";
            kernel = "../binaries/kernel/vmlinux.arm";
            machine = "virt";
        }
        Arch::Mips => {
            qemu = "qemu-system-mips";
            kernel = "../binaries/kernel/vmlinux.mips.2";
            machine = "malta";
        }
        Arch::Mipsel => {
            qemu = "qemu-system-mipsel";
            kernel = "../binaries/kernel/vmlinux.mipsel.4";
            machine = "";
        }
    }

    let mut process  = Command::new("sudo")
        .args(&[
            qemu,
            "-kernel", kernel,
            "-M", machine,
            "-hda", image,
            "-m", "256M",
            "-nographic",
            "-append", "root=/dev/sda1 rw init=preInit.sh",
            "-nic", "tap,ifname=tap-qemu,script=no,downscript=no"
        ])
        .stdin(Stdio::inherit()) // 允许向 QEMU 发送输入
        .stdout(Stdio::inherit()) // 捕获 QEMU 的输出
        .stderr(Stdio::inherit()) // 捕获 QEMU 的错误输出
        .spawn()
        .expect(&format!("Failed to execute command: {}", qemu));
    let _ = process.wait().expect("QEMU process wasn't running");
}
