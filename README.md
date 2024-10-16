# RustFAE

[TODO]
- clean mount and nbd connection  √
- raw image generation
- fix image DIR_825   80%
- emu image DIT_825   80%
- try openbmc : .mtd is massive, other .bin is only xz data √
- network between host and guest, need design the guest's init script
  - mode1: br0 in host (normal)         50%
  - mode2: br0 in guest (like FirmAE)
  - only tap    √
- try arm
  - compile arm kernel
  - emu R6300v2
- support configure tasks with toml file √