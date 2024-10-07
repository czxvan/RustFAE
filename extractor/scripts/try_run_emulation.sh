#! /bin/bash
# qemu-system-mips -M malta -kernel vmlinux-3.2.0-4-4kc-malta \
#     -hda test2.qcow2 -m 256M \
#     -append "root=/dev/sda1 rw init=/etc/rc.d/rcS" \
#     -nographic

qemu-system-mips -M malta -kernel vmlinux-3.2.0-4-4kc-malta \
    -hda image.qcow2 -m 256M \
    -append "root=/dev/sda1 rw init=/busybox.mipseb sh" \
    -nographic \
    -nic tap,ifname=tap-qemu,script=no,downscript=no
    # -nic tap,ifname=tap0,script=qemu-ifup,downscript=qemu-ifdown

    # -device e1000,netdev=net0 \
    # -netdev tap,id=net0,ifname=tap0,script=qemu-ifup,downscript=qemu-ifdown
    # -netdev tap,helper=/usr/lib/qemu/qemu-bridge-helper,id=hn0 \

    # -device e1000,netdev=n1,mac=52:54:00:12:34:56 \
    # -netdev user,id=n1,hostfwd=tcp::2280-:80,hostfwd=tcp::2443-:443,hostfwd=tcp::24817-:4817
    
