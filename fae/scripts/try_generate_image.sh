sudo modprobe nbd

sudo qemu-nbd -c /dev/nbd0 ../outputs/_DIR825B1_FW201EUB15.bin.extracted/image.qcow2

# 为/dev/nbd0创建分区，然后将/dev/nbd0p1格式化为ext4，并挂载到temp_image
echo -e "g\nn\n\n\n\nw" | sudo fdisk /dev/nbd0
sudo mkfs.ext4 /dev/nbd0p1
sudo mount /dev/nbd0p1 ../outputs/_DIR825B1_FW201EUB15.bin.extracted/temp_image

