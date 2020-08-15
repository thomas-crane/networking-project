use crate::client_state::ClientError;
use crate::client_state::ClientState;
use crate::lrdp_packet::LrdpPacket;

use log;
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::mpsc::{Receiver, RecvError, RecvTimeoutError, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// The minimum number of milliseconds which can elapse before a client will retransmit an
/// unacknowledged packet.
const RESEND_DELAY: u128 = 300;

/// The result which can be returned by a thread that the LRDP socket runs.
type ThreadResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// A buffer of data which has an address associated with it.
type AddressedBuffer = (Vec<u8>, SocketAddr);

/// A `T` which has been wrapped in an `Arc` and a `Mutex` so that it may be shared across threads.
type Shared<T> = Arc<Mutex<T>>;

/// Turns the `item` into a `Shared` version of itself.
fn shared<T>(item: T) -> Shared<T> {
    Arc::new(Mutex::new(item))
}

pub struct LrdpSocket {
    sender_tx: Sender<()>,
    reader_tx: Sender<Option<AddressedBuffer>>,
    udp_socket: UdpSocket,
    clients: Shared<HashMap<SocketAddr, ClientState>>,
    data_rx: Receiver<AddressedBuffer>,
}

impl LrdpSocket {
    pub fn bind<A: ToSocketAddrs>(addrs: A) -> std::io::Result<Self> {
        let udp_socket = UdpSocket::bind(addrs)?;
        let clients: Shared<HashMap<SocketAddr, ClientState>> = shared(HashMap::new());

        // set up channel for emitting data.
        let (data_tx, data_rx) = mpsc::channel::<AddressedBuffer>();

        // set up channel for stopping the reader thread.
        let (reader_tx, reader_rx) = mpsc::channel::<Option<AddressedBuffer>>();

        // start reading things from the socket. this thread just pulls data from the socket and
        // forwards it to the reader thread via the reader channel.
        let udp_reader_socket = udp_socket.try_clone()?;
        let udp_reader = reader_tx.clone();
        thread::spawn(move || {
            let mut buf = [0u8; std::u16::MAX as usize];
            let this_addr = udp_reader_socket.local_addr().unwrap().to_string();
            loop {
                let result = match udp_reader_socket.recv_from(&mut buf) {
                    Ok((recv, addr)) => {
                        log::debug!(
                            target: &this_addr,
                            "Received UDP packet from {}",
                            addr.to_string()
                        );
                        udp_reader.send(Some((buf[0..recv].to_vec(), addr)))
                    }
                    _ => udp_reader.send(None),
                };
                if result.is_err() {
                    log::warn!(
                        target: &this_addr,
                        "Error receiving UDP packet. Stopping reader."
                    );
                    break;
                }
            }
            log::trace!(target: &this_addr, "udp_reader thread at end.");
        });

        // set up the reader thread. This thread does the bulk of the processing work when a packet
        // is received.
        let reader_clients = clients.clone();
        let reader_socket = udp_socket.try_clone()?;
        thread::spawn(move || -> ThreadResult {
            let this_addr = reader_socket.local_addr().unwrap().to_string();
            loop {
                let read_result = reader_rx.recv()?;
                if read_result.is_none() {
                    log::warn!(
                        target: &this_addr,
                        "Got empty read result. Shutting down reader thread."
                    );
                    break;
                }
                let (buf, addr) = read_result.unwrap();

                // check if we know about this client yet.
                if !reader_clients.lock().unwrap().contains_key(&addr) {
                    log::info!(
                        target: &this_addr,
                        "... Adding new client {} from received data.",
                        addr.to_string()
                    );
                    // if not, create a new state for it.
                    let new_state = ClientState::new(addr);
                    reader_clients.lock().unwrap().insert(addr, new_state);
                }

                // if no data was received then this is a "closing" packet, so we can drop this client.
                if buf.len() == 0 {
                    log::info!(
                        target: &this_addr,
                        "... Client {} sent closing packet.",
                        addr.to_string()
                    );
                    reader_clients.lock().unwrap().remove(&addr);
                }

                log::debug!(
                    target: &this_addr,
                    "... Creating packet from header {:08b}",
                    buf[0]
                );
                let packet = LrdpPacket::from_buffer(&buf);

                // check if this packet is ACKing anything.
                if packet.has_ack() {
                    log::info!(
                        target: &this_addr,
                        "... ACK flag was set: {}",
                        packet.ack_num()
                    );
                    let mut clients = reader_clients.lock().unwrap();

                    // if the client sent us a bad value then drop it.
                    let ack_result = clients.get_mut(&addr).unwrap().ack(packet.ack_num());
                    if let Err(ClientError::WrongAck(_)) = ack_result {
                        log::error!(
                            target: &this_addr,
                            "... WrongAck {}. Dropping client",
                            packet.ack_num()
                        );
                        reader_clients.lock().unwrap().remove(&addr);
                    }
                }

                // check for any data.
                if packet.has_data() {
                    log::info!(
                        target: &this_addr,
                        "... DATA flag was set: {}",
                        packet.seq_num()
                    );
                    // check if the received sequence number is the expected one.
                    let mut clients = reader_clients.lock().unwrap();
                    let client = clients.get_mut(&addr).unwrap();

                    match client.recv(packet.seq_num()) {
                        Ok(_) => {
                            log::info!(target: &this_addr, "... Seq number OK, emitting data.");
                            // emit data. Don't really care about the result.
                            let _ = data_tx.send((packet.data().to_vec(), addr));
                            // ack the data.
                            let ack_packet =
                                LrdpPacket::create(Box::new([]), Some(packet.seq_num()), None);
                            reader_socket.send_to(ack_packet.as_buffer().as_slice(), addr)?;
                        }
                        // if the seq number is wrong then correct the sender.
                        Err(ClientError::WrongSeq(_, expected)) => {
                            log::warn!(
                                target: &this_addr,
                                "... Expected seq num {} but got {}, sending ack.",
                                expected,
                                packet.seq_num()
                            );
                            // send ack with expected number.
                            let ack_packet = LrdpPacket::create(Box::new([]), Some(expected), None);
                            reader_socket.send_to(ack_packet.as_buffer().as_slice(), addr)?;
                        }
                        // for any other error just drop this client.
                        Err(_) => {
                            log::error!(
                                target: &this_addr,
                                "... Other error occurred. Dropping client {}",
                                addr.to_string()
                            );
                            reader_clients.lock().unwrap().remove(&addr);
                        }
                    }
                }
            }
            log::trace!(target: &this_addr, "reader thread at end.");
            Ok(())
        });

        let (sender_tx, sender_rx) = mpsc::channel::<()>();
        // ack thread.
        let sender_clients = clients.clone();
        let sender_socket = udp_socket.try_clone()?;
        thread::spawn(move || -> ThreadResult {
            let this_addr = sender_socket.local_addr().unwrap().to_string();
            loop {
                match sender_rx.recv_timeout(Duration::from_millis(10)) {
                    Err(RecvTimeoutError::Timeout) => {}
                    _ => break,
                }
                // go through each client and check if any packets need to be retransmitted.
                let mut clients = sender_clients.lock().unwrap();
                for (addr, client) in clients.iter_mut() {
                    if client.last_send.map_or(false, |last_send| {
                        Instant::now().duration_since(last_send).as_millis() >= RESEND_DELAY
                    }) {
                        // resend last packet.
                        if let Some(packet) = client.next_packet() {
                            log::warn!(
                                target: &this_addr,
                                "Packet with seq {} was last sent more than {}ms ago. Sending again",
                                packet.seq_num(),
                                RESEND_DELAY
                            );
                            sender_socket
                                .send_to(packet.as_buffer().as_slice(), addr)
                                .unwrap();
                            client.last_send = Some(Instant::now());
                        }
                    }
                }
            }
            log::trace!(target: &this_addr, "sender thread at end.");
            Ok(())
        });

        Ok(Self {
            sender_tx,
            reader_tx,
            udp_socket,
            clients,
            data_rx,
        })
    }

    pub fn send_to<A: ToSocketAddrs>(
        &mut self,
        addr: A,
        data: &[u8],
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut addrs = addr.to_socket_addrs()?;
        let address = addrs.next().unwrap();
        let mut clients = self.clients.lock().unwrap();
        // check if we know about this client yet.
        if !clients.contains_key(&address) {
            log::info!(
                target: &self.udp_socket.local_addr().unwrap().to_string(),
                "Adding new client {} from sent data.",
                address.to_string()
            );
            // if not, create a new state for it.
            let new_state = ClientState::new(address);
            clients.insert(address, new_state);
        }
        let mut client = clients.get_mut(&address).unwrap();

        // queue the packet and send it.
        let packet = LrdpPacket::create(data.into(), None, Some(client.next_seq_num()));
        client.last_send = Some(Instant::now());
        self.udp_socket
            .send_to(packet.as_buffer().as_slice(), addr)
            .unwrap();
        client.enqueue(packet).unwrap();

        Ok(())
    }

    pub fn recv_from(&mut self) -> Result<AddressedBuffer, RecvError> {
        self.data_rx.recv()
    }

    pub fn stop(self) {
        log::info!(target: &self.udp_socket.local_addr().unwrap().to_string(), "Stopping socket...");
        // send stop messages over the channels.
        self.reader_tx.send(None).unwrap();
        self.sender_tx.send(()).unwrap();
        // explicitly drop the data_rx channel so that listeners are disconnected from it.
        drop(self.data_rx);
    }
}
