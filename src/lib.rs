pub mod azuki {

    use bincode::{deserialize, serialize};
    use serde::{Deserialize, Serialize};
    // use socket2::{Domain, Protocol, SockAddr, Socket, Type};
    use bitflags::bitflags;
    use std::array::IntoIter;
    use std::io::{self, BufRead, Read, Result, Write};
    use std::mem::MaybeUninit;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs, UdpSocket};
    use std::sync::Arc;
    use std::thread::{self, JoinHandle, Thread};

    use crate::switch;

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
            self.sock.send_to(data, self.peer.unwrap())
        }

        pub fn listen(
            &mut self,
            handler: impl Fn(&SocketAddr, &[u8], usize) + Send + Sync + 'static,
        ) -> Result<()> {
            let sock = self.sock.clone();
            let mut peer = self.peer.clone();
            let mut stat = self.status.clone();
            self.thread_handler = Some(thread::spawn(move || loop {
                let mut buff = [0; 1024];
                let (msg_size, new_peer) = sock.recv_from(&mut buff).unwrap(); // unpack
                let azuki = AzukiPack::unpack(&mut buff[..msg_size].to_vec());
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
                            };
                            sock.send_to(&buf.pack(), new_peer).expect(
                                format!("Unable to send SYNACK to {:?}", new_peer).as_str(),
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
                        if azuki.opt.contains(AzukiFlags::ACK){
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
                            sock.send_to(&buf.pack(), new_peer).expect(
                                format!("Unable to send SYNACK to {:?}", new_peer).as_str(),
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
                        if 
                    }
                    _ => {
                        continue;
                    }
                }
                // verify data
                handler(&peer.unwrap(), &azuki.dat, azuki.dat.len());
            }));
            Ok(())
        }
    }

    bitflags! {
        #[derive(Serialize, Deserialize)]
        struct AzukiFlags: u16 {
            const KEEP_ALIVE = 0b10000000;
            const SYN = 0b00000001;
            const ACK = 0b00000010;
            const FIN = 0b00000100;
            const RST = 0b00001000;
            const SEG = 0b00010000; // sagmentation bit, if set, the packet has following packet(s)
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct AzukiPack {
        ver: u8,
        seq: u32,
        opt: AzukiFlags,
        #[serde(with = "serde_bytes")]
        dat: Vec<u8>,
    }
    trait AzukiDeserialize {
        fn unpack(&self) -> AzukiPack;
    }

    impl AzukiPack {
        fn unpack(msg: &mut Vec<u8>) -> Self {
            bincode::deserialize::<AzukiPack>(msg).unwrap()
        }
        fn pack(self) -> Vec<u8> {
            bincode::serialize(&self).unwrap()
        }
    }

    impl AzukiDeserialize for Vec<u8> {
        fn unpack(&self) -> AzukiPack {
            bincode::deserialize::<AzukiPack>(self).unwrap()
        }
    }
}

