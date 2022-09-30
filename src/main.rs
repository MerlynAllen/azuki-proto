use azukiproto::azuki::Azuki;
use std::io::{self, BufRead, Result, Write};
use std::net::ToSocketAddrs;
fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut local_addr_str = String::new();
    print!("Bind Addr:");
    stdout.flush().unwrap();
    stdin.lock().read_line(&mut local_addr_str)?;
    let local_addr_str = local_addr_str.strip_suffix("\n").unwrap();
    let local_addr = local_addr_str
        .to_socket_addrs()
        .unwrap_or_else(|e| {
            eprintln!("SocketAddr invalid!");
            panic!("Invalid SocketAddr");
        })
        .next()
        .unwrap();
    println!("Binded at {:?}", local_addr);
    // let mut peer_addr_str = String::new();
    // print!("Remote Addr:");
    // stdout.flush().unwrap();
    // stdin.lock().read_line(&mut peer_addr_str)?;
    // let peer_addr_str = peer_addr_str.strip_suffix("\n").unwrap();
    // let peer_addr = peer_addr_str
    //     .to_socket_addrs()
    //     .unwrap_or_else(|e| {
    //         eprintln!("SocketAddr invalid!");
    //         panic!("Invalid SocketAddr");
    //     })
    //     .next()
    //     .unwrap();
    // println!("Send to remote: {:?}", peer_addr);

    // init socket
    let mut azuki = Azuki::bind(local_addr.ip(), local_addr.port())
        .map_err(|_e| {
            eprintln!("Bind failed!");
            panic!("Bind failed!");
        })
        .unwrap();
    // azuki.connect(peer_addr.ip(), peer_addr.port())?;
    azuki.listen(|peer, data, size| {
        println!(
            "Received {} bytes from {:?}: {}",
            size,
            peer,
            String::from_utf8_lossy(data)
        );
    })?;
    azuki.thread_handler.unwrap().join().unwrap();
    Ok(())
}