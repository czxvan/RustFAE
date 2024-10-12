
/// Clean functions
pub use crate::generator::utils::umount_temp_images;
pub use crate::generator::utils::disconnect_nbd_divices;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Arch {
    Arm,
    Mips,
    Mipsel,
}

impl std::str::FromStr for Arch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "arm" => Ok(Arch::Arm),
            "mips" => Ok(Arch::Mips),
            "mipsel" => Ok(Arch::Mipsel),
            _ => Err(format!("Invalid arch: {}", s))
        }
    }
}
impl Arch {
    pub fn to_str(&self) -> &str {
        match self {
            Arch::Arm => "arm",
            Arch::Mips => "mips",
            Arch::Mipsel => "mipsel"
        }
    }
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ImageType {
    Qcow2,
    #[allow(dead_code)]
    Raw
}

impl std::str::FromStr for ImageType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "qcow2" => Ok(ImageType::Qcow2),
            "raw" => Ok(ImageType::Raw),
            _ => Err(format!("Invalid image type: {}", s))
        }
    }
}