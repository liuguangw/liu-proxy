use super::{AddressType, ParseAddressTypeError};
use bytes::{BufMut, Bytes, BytesMut};
use std::{
    array::TryFromSliceError,
    fmt,
    net::{IpAddr, Ipv4Addr},
    ops::Range,
    str::Utf8Error,
};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};

///连接目标地址
pub enum ConnDestAddr {
    ///IP地址
    Ip(IpAddr),
    ///域名
    Domain(String),
}
///目标地址和端口信息
pub struct ConnDest {
    pub addr: ConnDestAddr,
    pub port: u16,
}

impl Default for ConnDest {
    ///构造一个默认的地址和端口信息
    fn default() -> Self {
        let default_ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
        Self {
            addr: ConnDestAddr::Ip(default_ip),
            port: 80,
        }
    }
}

///解析目标地址和端口出错
#[derive(Error, Debug)]

pub enum ParseConnDestError {
    #[error("{0} out of range, index={1}, length={2}")]
    OutofRange(String, usize, usize),
    #[error("parse ip as slice failed: {0}")]
    ParseIpSlice(#[from] TryFromSliceError),
    #[error("parse domain as utf-8 string failed: {0}")]
    ParseDomainStr(#[from] Utf8Error),
    #[error("parse address type failed: {0}")]
    ParseAddressType(#[from] ParseAddressTypeError),
    #[error("read stream failed: {0}")]
    IoErr(#[from] std::io::Error),
}

impl ParseConnDestError {
    pub fn new_out_of_range(attribute: &str, index: usize, length: usize) -> Self {
        Self::OutofRange(attribute.to_string(), index, length)
    }
}

impl ConnDest {
    //计算地址长度
    fn get_addr_length(
        value: &[u8],
        value_length: usize,
        addr_type: &AddressType,
    ) -> Result<u8, ParseConnDestError> {
        let addr_length = match addr_type {
            AddressType::IpV4 => 4,
            AddressType::IpV6 => 16,
            AddressType::Domain => {
                //域名
                let pos = 1;
                if pos >= value_length {
                    return Err(ParseConnDestError::new_out_of_range(
                        "domain length",
                        pos,
                        value_length,
                    ));
                } else {
                    value[pos]
                }
            }
        };
        Ok(addr_length)
    }
    ///解析地址
    fn parse_addr(
        value: &[u8],
        value_length: usize,
        range: Range<usize>,
        addr_type: &AddressType,
    ) -> Result<ConnDestAddr, ParseConnDestError> {
        //读取buffer
        let addr_buffer = {
            let buffer_last_pos = range.end - 1;
            if buffer_last_pos >= value_length {
                return Err(ParseConnDestError::new_out_of_range(
                    "addr_buffer",
                    buffer_last_pos,
                    value_length,
                ));
            } else {
                &value[range]
            }
        };
        let addr = match addr_type {
            AddressType::IpV4 => {
                let buff: [u8; 4] = addr_buffer.try_into()?;
                ConnDestAddr::Ip(IpAddr::from(buff))
            }
            AddressType::IpV6 => {
                let buff: [u8; 16] = addr_buffer.try_into()?;
                ConnDestAddr::Ip(IpAddr::from(buff))
            }
            AddressType::Domain => {
                let domain = std::str::from_utf8(addr_buffer)?;
                ConnDestAddr::Domain(domain.to_string())
            }
        };
        Ok(addr)
    }

    /// 解析客户端传入的目标地址和端口
    ///
    /// ```
    /// +------+----------+----------+
    /// | ATYP | DST.ADDR | DST.PORT |
    /// +------+----------+----------+
    /// |  1   | Variable |    2     |
    /// +------+----------+----------+
    /// ```
    /// - ATYP 目标地址类型，有如下取值：
    ///   - 0x01 IPv4
    ///   - 0x03 域名
    ///   - 0x04 IPv6
    /// - DST.ADDR 目标地址
    ///   - 如果是IPv4，那么就是4 bytes
    ///   - 如果是IPv6那么就是16 bytes
    ///   - 如果是域名，那么第一个字节代表字符长度, 接下来的数据是目标地址
    /// - DST.PORT 两个字节代表端口号
    pub fn try_from_bytes(value: &[u8]) -> Result<Self, ParseConnDestError> {
        let mut pos = 0;
        let value_length = value.len();
        let addr_type = if pos >= value_length {
            return Err(ParseConnDestError::new_out_of_range(
                "addr type",
                pos,
                value_length,
            ));
        } else {
            AddressType::try_from(value[pos]).map_err(ParseConnDestError::ParseAddressType)?
        };
        pos += 1;
        //计算地址长度
        let addr_length = Self::get_addr_length(value, value_length, &addr_type)?;
        if addr_type.is_domain() {
            //域名长度读取了一个字节
            pos += 1;
        }
        let addr_range = pos..pos + (addr_length as usize);
        let addr = Self::parse_addr(value, value_length, addr_range, &addr_type)?;
        pos += addr_length as usize;
        //port
        let port_last_index = pos + 1;
        let port = if port_last_index >= value_length {
            return Err(ParseConnDestError::new_out_of_range(
                "port type",
                port_last_index,
                value_length,
            ));
        } else {
            let mut port_value = (value[pos] as u16) << 8;
            port_value += value[port_last_index] as u16;
            port_value
        };
        Ok(Self { addr, port })
    }

    pub async fn try_from_stream<T>(stream: &mut T) -> Result<Self, ParseConnDestError>
    where
        T: AsyncRead + Unpin,
    {
        let addr_type = stream.read_u8().await?;
        let addr_type =
            AddressType::try_from(addr_type).map_err(ParseConnDestError::ParseAddressType)?;
        let addr = match addr_type {
            AddressType::IpV4 => {
                let mut buff = [0; 4];
                stream.read_exact(&mut buff).await?;
                ConnDestAddr::Ip(IpAddr::from(buff))
            }
            AddressType::IpV6 => {
                let mut buff = [0; 16];
                stream.read_exact(&mut buff).await?;
                ConnDestAddr::Ip(IpAddr::from(buff))
            }
            AddressType::Domain => {
                let buff_size = stream.read_u8().await?;
                let mut buff = vec![0; buff_size as usize];
                stream.read_exact(&mut buff).await?;
                let domain = std::str::from_utf8(&buff)?;
                ConnDestAddr::Domain(domain.to_string())
            }
        };
        let port = stream.read_u16().await?;
        Ok(Self { addr, port })
    }

    pub fn to_raw_data(&self) -> Bytes {
        let (addr_type, addr_buffer) = match &self.addr {
            ConnDestAddr::Ip(ip) => match ip {
                IpAddr::V4(s) => (AddressType::IpV4, s.octets().to_vec()),
                IpAddr::V6(s) => (AddressType::IpV6, s.octets().to_vec()),
            },
            ConnDestAddr::Domain(s) => {
                let mut buff = Vec::with_capacity(s.len() + 1);
                buff.push(s.len() as u8);
                buff.extend_from_slice(s.as_bytes());
                (AddressType::Domain, buff)
            }
        };
        let mut raw_data = BytesMut::with_capacity(addr_buffer.len() + 3);
        raw_data.put_u8(addr_type.into());
        raw_data.put_slice(&addr_buffer);
        raw_data.put_u16(self.port);
        raw_data.into()
    }
}

impl fmt::Display for ConnDest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let addr = match &self.addr {
            ConnDestAddr::Ip(ip) => ip.to_string(),
            ConnDestAddr::Domain(domain) => domain.to_string(),
        };
        write!(f, "{}:{}", addr, self.port)
    }
}
