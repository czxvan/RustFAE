pub fn init_network() {
    let tap_name = "tap-qemu";
    let ip_addr = "192.168.1.1/24";
    // Create tap
    let output = std::process::Command::new("sudo")
        .args(&["bash", "-c"])
        .arg(&format!("ip tuntap add {} mode tap", tap_name))
        .output()
        .expect("Failed to execute command: ip tuntap");

    if !output.status.success() {
        eprintln!("tuntap: {} already exist", tap_name);
    } else {
        println!("Create tuntap: {}", tap_name)
    }

    // Set ip address for tap
    let output = std::process::Command::new("sudo")
        .args(&["bash", "-c"])
        .arg(&format!("ip add add {} dev {}", ip_addr, tap_name))
        .output()
        .expect("Failed to execute command: ip addr");

    if !output.status.success() {
        eprintln!("Ip addr {} already exist for {}", ip_addr, tap_name);
    } else {
        println!("Set ip address for {}", tap_name);
    }

    // Set tap up
    let output = std::process::Command::new("sudo")
        .args(&["bash", "-c"])
        .arg(&format!("ip link set {} up", tap_name))
        .output()
        .expect("Failed to execute command: ip link");

    if !output.status.success() {
        eprintln!("Failed to set set eth0 up: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    } else {
        println!("Set {} up", tap_name);
    }
}