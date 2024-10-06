#! /bin/bash
# qemu-system-mips -M malta -kernel vmlinux-3.2.0-4-4kc-malta \
#     -hda test2.qcow2 -m 256M \
#     -append "root=/dev/sda1 rw init=/etc/rc.d/rcS" \
#     -nographic

qemu-system-mips -M malta -kernel vmlinux-3.2.0-4-4kc-malta \
    -hda image.qcow2 -m 256M \
    -append "root=/dev/sda1 rw init=/busybox.mipseb sh" \
    -nographic
