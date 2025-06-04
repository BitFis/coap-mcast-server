use coap_lite::{CoapRequest, Packet};

use std::env;
use std::net::Ipv6Addr;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let parts: Vec<&str> = args[1].split('%').collect();

    let mcastaddr = &parts[0];
    let mut scope = 0;
    if parts.len() > 1 {
        match parts[1].parse::<u32>() {
            Ok(number) => scope = number,
            Err(e) => println!("Failed to convert '{}' to number: {}", parts[1], e),
        }
    }

    let mut same = false;
    let mut diff = true;
    if args.len() > 2 {
        let input: &str = &args[2];
        match input {
            "both" => {
                same = true;
                diff = true
            }
            "same" => {
                same = true;
                diff = false
            }
            "diff" => {
                same = false;
                diff = true
            }
            _ => println!("Test arg invalid, expected [both|same|diff]."),
        }
    }

    let socket = tokio::net::UdpSocket::bind("[::]:5683").await?;
    let mcast = mcastaddr
        .parse::<Ipv6Addr>()
        .expect("Failed to parse ipv6 addr"); // Ipv6Addr::new(0xff05, 0, 0, 0, 0, 0, 0, 0x1234);
    socket.set_multicast_loop_v6(true).unwrap();
    socket
        .join_multicast_v6(&mcast, scope)
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

        let mut response = request.response.unwrap();
        response.message.payload = b"OK".to_vec();

        // respond from different port
        let packet = response.message.to_bytes().unwrap();
        if diff {
            respsocket
                .send_to(&packet[..], &addr)
                .await
                .expect("Failed to send response");
        }

        // respond from same port
        if same {
            socket
                .send_to(&packet[..], &addr)
                .await
                .expect("Failed to send response");
        }
    }
}
