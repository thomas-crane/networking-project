use std::collections::VecDeque;
use std::io::Result;
use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;

const FLAG_MASK: u8 = 0b11000000;
const ID_MASK: u8 = 0b00111111;
const IMPORTANT_FLAG: u8 = 0b10000000;
const ACK_FLAG: u8 = 0b01000000;

struct ReadonlySocket(UdpSocket);
impl ReadonlySocket {
    pub fn new(socket: UdpSocket) -> Self {
        Self(socket)
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        self.0.recv_from(buf)
    }
}

struct WriteonlySocket(UdpSocket);
impl WriteonlySocket {
    pub fn new(socket: UdpSocket) -> Self {
        Self(socket)
    }

    pub fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addrs: A) -> Result<usize> {
        self.0.send_to(buf, addrs)
    }
}

pub struct SrdpSocket {
    socket: Arc<Mutex<WriteonlySocket>>,
    read_socket: ReadonlySocket,
    available_ids: VecDeque<u8>,
    // ack stuff
    unacked_packets: Arc<Mutex<VecDeque<UnackedPacket>>>,
}

impl SrdpSocket {
    pub fn bind<A: ToSocketAddrs>(addrs: A) -> Result<Self> {
        let read_socket = UdpSocket::bind(addrs)?;
        let ack_listener = read_socket.try_clone()?;
        let write_socket = read_socket.try_clone()?;
        let read_socket = ReadonlySocket::new(read_socket);
        let write_socket = WriteonlySocket::new(write_socket);

        let socket = Arc::new(Mutex::new(write_socket));

        // thread to watch for unacked packets and send them again if needed.

        let mut available_ids = VecDeque::with_capacity(64);
        for i in 0..64u8 {
            available_ids.push_front(i);
        }

        let unacked_packets = Arc::new(Mutex::new(VecDeque::with_capacity(64)));
        let unacked_clone = unacked_packets.clone();
        let unacked_socket = socket.clone();
        thread::spawn(move || {
            // TODO: this hurts me.
            thread::sleep(std::time::Duration::from_millis(100));

            // try to get the first unacked packet.
            let mut unacked = unacked_clone.lock().unwrap();
            let packet: Option<UnackedPacket> = unacked.pop_front();
            // if there is a packet, convert it to bytes and send it.
            if let Some(unacked_packet) = packet {
                let bytes = unacked_packet.as_bytes().into_boxed_slice();
                let socket = unacked_socket.lock().unwrap();
                socket.send_to(&bytes, unacked_packet.2).unwrap();
                // requeue the packet.
                unacked.push_back(unacked_packet);
            }
        });

        let ack_listener_unacked_clone = unacked_packets.clone();
        thread::spawn(move || {
            let mut buf = [0];
            loop {
                if let Ok(1) = ack_listener.peek(&mut buf) {
                    let flags = buf[0] & FLAG_MASK;
                    // if the header was an ack, remove the acknowledged packet and keep looping.
                    if (flags & ACK_FLAG) == ACK_FLAG {
                        // receive the datagram.
                        ack_listener.recv(&mut buf).unwrap();
                        let id = buf[0] & ID_MASK;
                        let mut unacked_packets = ack_listener_unacked_clone.lock().unwrap();
                        let unacked = unacked_packets.iter().position(|p| p.0 == id);
                        match unacked {
                            Some(unacked_id) => {
                                unacked_packets.remove(unacked_id);
                            }
                            None => {
                                panic!();
                            },
                        }
                    }
                }
            }
        });

        Ok(Self {
            socket,
            read_socket,
            available_ids,
            unacked_packets,
        })
    }

    pub fn recv(&mut self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        let mut header = [0u8];
        loop {
            // receive the header.
            if let Ok((1, addr)) = self.read_socket.recv_from(&mut header) {
                let flags = header[0] & FLAG_MASK;

                // if the header was an ack, remove the acknowledged packet and keep looping.
                if (flags & ACK_FLAG) == ACK_FLAG {
                    let id = header[0] & ID_MASK;
                    let mut unacked_packets = self.unacked_packets.lock().unwrap();
                    let unacked = unacked_packets.iter().position(|p| p.0 == id);
                    match unacked {
                        Some(unacked_id) => {
                            unacked_packets.remove(unacked_id);
                            self.available_ids.push_back(id);
                        }
                        None => panic!(),
                    }
                    continue;
                }

                // if the header was an important message, send an ack.
                if (flags & IMPORTANT_FLAG) == IMPORTANT_FLAG {
                    // create an ack.
                    let ack = [ACK_FLAG | (header[0] & ID_MASK)];
                    self.socket.lock().unwrap().send_to(&ack, addr)?;
                }

                // stop looping since we can now receive data.
                break;
            } else {
                panic!();
            }
        }

        // recv into the buffer.
        self.read_socket.recv_from(buf)
    }

    pub fn send_to(&mut self, packet: &Packet, addr: SocketAddr) -> Result<usize> {
        match packet {
            Packet::Normal(data) => self.socket.lock().unwrap().send_to(data, addr),
            Packet::Important(data) => {
                // acquire an ID for the packet.
                match self.available_ids.pop_front() {
                    Some(id) => {
                        let unacked = UnackedPacket::new(id, data.clone(), addr);
                        // send the packet.
                        let bytes = unacked.as_bytes();
                        let result = self.socket.lock().unwrap().send_to(&bytes, addr);

                        // enqueue the packet.
                        let mut unacked_packets = self.unacked_packets.lock().unwrap();
                        unacked_packets.push_back(unacked);

                        result
                    }
                    None => {
                        // TODO.
                        Err(Error::new(ErrorKind::WouldBlock, "No available ids."))
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct UnackedPacket(u8, Box<[u8]>, SocketAddr);

impl UnackedPacket {
    pub fn new(id: u8, data: Box<[u8]>, addr: SocketAddr) -> Self {
        Self(id, data, addr)
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let header = [(self.0 & ID_MASK) | IMPORTANT_FLAG];
        [&header, &*self.1].concat()
    }
}

pub enum Packet {
    Important(Box<[u8]>),
    Normal(Box<[u8]>),
}
