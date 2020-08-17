use std::thread;
use std::time::Duration;
use protocol::lrdp_socket::LrdpSocket;
use log;
use pretty_env_logger;

fn main() {
    pretty_env_logger::init();
    let sender = thread::spawn(|| {
        thread::sleep(Duration::from_secs(1));
        log::info!("Creating sender socket");
        let mut socket = LrdpSocket::bind("127.0.0.1:0").unwrap();
        for _ in 0..3 {
            thread::sleep(Duration::from_secs(1));
            log::info!("Sending...");
            socket.send_to("127.0.0.1:6860", &[1, 2, 3]).unwrap();
        }
        socket.stop();
    });

    let receiver = thread::spawn(|| {
        let mut socket = LrdpSocket::bind("127.0.0.1:6860").unwrap();

        for _ in 0..4 {
            let (buf, addr) = socket.recv_from().unwrap();
            log::info!("Received some data from {}: {:?}", addr.to_string(), buf);
        }
        socket.stop();
    });

    sender.join().unwrap();
    log::info!("Sender joined");
    receiver.join().unwrap();
    log::info!("Receiver joined");

    thread::sleep(Duration::from_secs(1));
    log::info!("Shutting down test.");
}
