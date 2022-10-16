//    use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
// use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use bitflags::bitflags;
use log::{debug, error, info, trace, warn};
use std::array::IntoIter;
use std::io::{self, BufRead, Read, Result, Write};
use std::mem::MaybeUninit;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::Arc;
use std::thread::{self, JoinHandle, Thread};

use crate::azuki::*;
bitflags! {
    #[derive(Serialize, Deserialize)]
pub (crate) struct AzukiFlags: u16 {
        const KEEP_ALIVE = 0b10000000;
        const SYN = 0b00000001;
        const ACK = 0b00000010;
        const FIN = 0b00000100;
        const RST = 0b00001000;
        const SEG = 0b00010000; // sagmentation bit, if set, the packet has following packet(s)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct AzukiPack {
    pub(crate) ver: u8,
    pub(crate) seq: u32,
    pub(crate) opt: AzukiFlags,
    #[serde(with = "serde_bytes")]
    pub(crate) dat: Vec<u8>,
}
pub(crate) trait AzukiDeserialize {
    fn unpack(&self) -> Result<AzukiPack>;
}

impl AzukiPack {
    pub(crate) fn unpack(msg: &mut Vec<u8>) -> Result<AzukiPack> {
        bincode::deserialize::<AzukiPack>(msg).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
    pub(crate) fn pack(self) -> Result<Vec<u8>> {
        bincode::serialize(&self).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

impl AzukiDeserialize for Vec<u8> {
    fn unpack(&self) -> Result<AzukiPack> {
        bincode::deserialize::<AzukiPack>(self).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}
