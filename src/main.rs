use coap_lite::{CoapRequest, Packet};

use std::env;
use std::net::Ipv6Addr;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mcastaddr = &args[1];

    let socket = tokio::net::UdpSocket::bind("[::]:5683").await?;
    let mcast = mcastaddr.parse::<Ipv6Addr>().expect("Failed to parse ipv6 addr"); // Ipv6Addr::new(0xff05, 0, 0, 0, 0, 0, 0, 0x1234);
    socket.set_multicast_loop_v6(true).unwrap();
    socket
        .join_multicast_v6(&mcast, 0)
        .expect("failed to join multicast address");

    let respsocket = tokio::net::UdpSocket::bind("[::]:55660").await?;

    let mut buf = [0; 1024];
    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;

        let packet = Packet::from_bytes(&buf[..len]).unwrap();
        let request = CoapRequest::from_packet(packet, addr);

        let method = request.get_method().clone();
        let path = request.get_path();

        println!(
            "Received CoAP request '{:?} /{}' from {}",
            method, path, addr
        );

        // respond from different port
        let mut response = request.response.unwrap();
        response.message.payload = b"OK".to_vec();

        let packet = response.message.to_bytes().unwrap();

        respsocket
            .send_to(&packet[..], &addr)
            .await
            .expect("Failed to send response");
    }
}
