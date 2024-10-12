#! /busybox sh
BUSYBOX="/busybox"
${BUSYBOX} ip addr add 192.168.1.2/24 dev eth0
${BUSYBOX} ip link set eth0 up

# [TODO!]
# BUSYBOX=/firmadyne/busybox

# [ -d /dev ] || mkdir -p /dev
# [ -d /root ] || mkdir -p /root
# [ -d /sys ] || mkdir -p /sys
# [ -d /proc ] || mkdir -p /proc
# [ -d /tmp ] || mkdir -p /tmp
# mkdir -p /var/lock

# ${BUSYBOX} mount -t sysfs sysfs /sys
# ${BUSYBOX} mount -t proc proc /proc
# ${BUSYBOX} ln -sf /proc/mounts /etc/mtab

# mkdir -p /dev/pts
# ${BUSYBOX} mount -t devpts devpts /dev/pts
# ${BUSYBOX} mount -t tmpfs tmpfs /run

# /sbin/preinit &

# /firmadyne/network.sh &
# /firmadyne/run_service.sh &
# /firmadyne/debug.sh
# /firmadyne/busybox sleep 36000