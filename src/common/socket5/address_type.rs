use thiserror::Error;

///目标地址类型
pub enum AddressType {
    IpV4,
    IpV6,
    Domain,
}

#[derive(Error, Debug)]
#[error("invalid address type {0}")]
pub struct ParseAddressTypeError(pub u8);

impl From<AddressType> for u8 {
    fn from(value: AddressType) -> Self {
        match value {
            AddressType::IpV4 => 1,
            AddressType::IpV6 => 4,
            AddressType::Domain => 3,
        }
    }
}

impl TryFrom<u8> for AddressType {
    type Error = ParseAddressTypeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 1 {
            Ok(Self::IpV4)
        } else if value == 4 {
            Ok(Self::IpV6)
        } else if value == 3 {
            Ok(Self::Domain)
        } else {
            Err(ParseAddressTypeError(value))
        }
    }
}

impl AddressType {
    pub fn is_domain(&self) -> bool {
        matches!(self, Self::Domain)
    }
}
