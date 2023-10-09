use std::net::{Ipv4Addr, SocketAddrV4};

pub const DEFAULT_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 32_111);

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
