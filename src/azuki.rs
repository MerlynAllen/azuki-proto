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

pub use crate::azuki_pack::*;

bitflags! {
    struct AzukiStatus: u16 {
        const NULL = 0b00000000;
        const ESTABLISHED = 0b00000001;
        const SYN_SENT = 0b00000010;
        const SYN_RECV = 0b00000100;
        const FIN_WAIT1 = 0b00001000;
        const FIN_WAIT2 = 0b00010000;
        const TIME_WAIT = 0b00100000;
        const CLOSE = 0b01000000;
        const CLOSE_WAIT = 0b10000000;
    }
}
pub const AZUKI_MTU: usize = 1240;
pub struct Azuki {
    peer: Arc<Option<SocketAddr>>,
    sock: Arc<UdpSocket>,
    curr_serialno: u32, // max 4294967295
    status: Arc<AzukiStatus>,
    window_size: u16, // 65535 currently
    pub thread_handler: Option<JoinHandle<()>>,
}

impl Azuki {
    pub fn bind(host: IpAddr, port: u16) -> Result<Self> {
        let socket_serialize_res = UdpSocket::bind(SocketAddr::new(host, port));
        match socket_serialize_res {
            Ok(socket) => {
                let azuki = Azuki {
                    peer: Arc::new(None),
                    sock: Arc::new(socket),
                    curr_serialno: 0,
                    status: Arc::new(AzukiStatus::NULL),
                    window_size: 65535,
                    thread_handler: None,
                };
                Ok(azuki)
            }
            Err(e) => Err(e),
        }
    }
    pub fn connect(&mut self, host: IpAddr, port: u16) -> Result<()> {
        self.peer = Some(SocketAddr::new(host, port)).into();
        /* hand shake */
        Ok(())
    }
    pub fn send(&mut self, data: &Vec<u8>) -> Result<usize> {
        // TODO: implement
        let azuki = AzukiPack {
            ver: 0,
            seq: self.curr_serialno,
            opt: AzukiFlags::SYN,
            dat: data.clone(),
        };
        let azuki = azuki.pack().expect("failed to pack azuki");
        self.sock.send_to(&azuki, self.peer.unwrap())
    }

    pub fn listen(
        &mut self,
        handler: impl Fn(&SocketAddr, &[u8], usize) + Send + Sync + 'static,
    ) -> Result<()> {
        let sock = self.sock.clone();
        let mut peer = self.peer.clone();
        let mut stat = self.status.clone();
        self.thread_handler = Some(
            thread::Builder::new()
                .name("Listener".to_string())
                .spawn(move || loop {
                    let mut buff = [0; 1024];
                    let sock_recv = sock.recv_from(&mut buff);
                    let (msg_size, new_peer) = match sock_recv {
                        Err(e) => {
                            match e.raw_os_error() {
                                Some(os_err_no) => {
                                    match os_err_no {
                                        10040 => {
                                            // windows: length exceeded
                                            error!(
                                        "Package length exceeded MTU. Current MTU is {AZUKI_MTU}."
                                    );
                                            continue;
                                        }
                                        _ => {
                                            panic!("Unknown error number {os_err_no}! {e:?}");
                                        }
                                    }
                                }
                                None => panic!("Unexpected error! Error: {e:?}"),
                            }
                        }
                        Ok(result) => result,
                    };
                    let azuki = AzukiPack::unpack(&mut buff[..msg_size].to_vec());
                    let azuki = match azuki {
                        Ok(azuki) => azuki,
                        Err(e) => {
                            error!("Package corrupted! Error: {e:?}");
                            continue;
                        }
                    };
                    // filter by peer sock
                    match stat.as_ref().clone() {
                        AzukiStatus::NULL => {
                            // new sock
                            if azuki.opt.contains(AzukiFlags::SYN) {
                                // server passive receive SYN
                                // reply SYNACK
                                let buf = AzukiPack {
                                    ver: 1,
                                    seq: 0,
                                    opt: AzukiFlags::SYN | AzukiFlags::ACK,
                                    dat: vec![],
                                }
                                .pack();
                                let buf = match buf {
                                    Ok(buf) => buf,
                                    Err(e) => {
                                        error!("Package corrupted! Error: {e:?}");
                                        continue;
                                    }
                                };
                                sock.send_to(&buf, new_peer).expect(
                                    format!("Unable to send SYNACK to {new_peer:?}").as_str(),
                                );
                                // wait for establish
                                peer = Some(new_peer).into();
                                stat = AzukiStatus::SYN_RECV.into();
                            }
                            continue;
                        }
                        AzukiStatus::SYN_SENT => {
                            // client sent SYN
                            // in this state, peer sock info should already been set. Some(peer) value is guaranteed.
                            // first filter by peer sock
                            if new_peer != peer.unwrap() {
                                continue;
                            }
                            // check ACK
                            if azuki.opt.contains(AzukiFlags::ACK) {
                                // established when meet ACK
                                // TODO: add a delay
                                stat = AzukiStatus::ESTABLISHED.into();
                            }
                            if azuki.opt.contains(AzukiFlags::SYN) {
                                // client receive SYN and reply ack
                                let buf = AzukiPack {
                                    ver: 1,
                                    seq: 0,
                                    opt: AzukiFlags::ACK,
                                    dat: vec![],
                                };
                                let buf = match buf.pack() {
                                    Ok(buf) => buf,
                                    Err(e) => {
                                        error!("Package corrupted! Error: {e:?}");
                                        continue;
                                    }
                                };
                                sock.send_to(&buf, new_peer).expect(
                                    format!("Unable to send SYNACK to {new_peer:?}").as_str(),
                                );
                            }
                            continue;
                        }
                        AzukiStatus::SYN_RECV => {
                            // server sent SYN + ACK
                            if azuki.opt.contains(AzukiFlags::ACK) {
                                // server wait for ACK
                                // maybe add a delay
                                stat = AzukiStatus::ESTABLISHED.into();
                            }
                        }
                        AzukiStatus::ESTABLISHED => {
                            // peer sock info should already been set. Some(peer) value is guaranteed.
                            // first filter by peer sock
                            if new_peer != peer.unwrap() {
                                continue;
                            }
                            // filter by packet opt
                        }
                        _ => {
                            continue;
                        }
                    }
                    // verify data
                    handler(&peer.unwrap(), &azuki.dat, azuki.dat.len());
                })
                .expect("Unable to spawn listener thread!"),
        );
        Ok(())
    }
}
