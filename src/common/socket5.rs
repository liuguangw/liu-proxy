mod  address_type;
mod conn_dest;
pub use address_type::{AddressType,ParseAddressTypeError};
pub use conn_dest::{ConnDest,ParseConnDestError};
///版本
pub const VERSION:u8 =5;
