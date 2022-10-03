mod address_type;
mod build_response_ns;
mod conn_dest;

pub use address_type::{AddressType, ParseAddressTypeError};
pub use build_response_ns::build_response;
pub use conn_dest::{ConnDest, ParseConnDestError};
///版本
pub const VERSION: u8 = 5;
