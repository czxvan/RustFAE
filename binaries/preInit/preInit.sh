#! /busybox sh
BUSYBOX="/busybox"

[ -d /dev ] || mkdir -p /dev
[ -d /root ] || mkdir -p /root
[ -d /sys ] || mkdir -p /sys
[ -d /proc ] || mkdir -p /proc
[ -d /tmp ] || mkdir -p /tmp
${BUSYBOX} mkdir -p /var/lock

${BUSYBOX} mount -t sysfs sysfs /sys
${BUSYBOX} mount -t proc proc /proc
${BUSYBOX} ln -sf /proc/mounts /etc/mtab

mkdir -p /dev/pts
${BUSYBOX} mount -t devpts devpts /dev/pts
${BUSYBOX} mount -t tmpfs tmpfs /run

${BUSYBOX} ip addr add 192.168.1.2/24 dev eth0
${BUSYBOX} ip link set eth0 up

/busybox sh