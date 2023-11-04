use std::collections::HashMap;
use std::net::{Ipv4Addr, UdpSocket};
use trust_dns_proto::{
    op::Message,
    serialize::binary::{BinDecodable, BinDecoder},
};

#[tokio::main]
async fn main() {
    // listen for incoming udp packets
    let socket = UdpSocket::bind("127.0.0.1:53")
        .expect("Could not bind client socket");


    // cache hashmap
    let mut cache = HashMap::new();


    loop {
        handle_query(&socket, &mut cache).await;
    }
}

async fn handle_query(socket: &UdpSocket, cache: &mut HashMap<String, Ipv4Addr>) {
    // buffer to store incoming data
    let mut buffer = [0; 512];

    // receive data from socket
    let (number_of_bytes, src_addr) = socket.recv_from(&mut buffer)
        .expect("Didn't receive data");

    // log
    println!("Received {} bytes from {}", number_of_bytes, src_addr);


    let start_timestamp = std::time::Instant::now();

    // parse the dns request
    let parsed_req = parse_dns_request(&buffer[..number_of_bytes]);

    // Extract the number of questions and answers from Header
    let message = parsed_req.expect("Error parsing dns request");

    // Loop over all questions : get the domain name, record type and class
    let mut questions = Vec::new();
    for i in 0..message.header().query_count() {
        let question = message
            .clone()
            .queries()[i as usize]
            .clone();
        questions.push(question);
    }

    // log domain name, record type and class
    // println!("Domain name: {}", questions[0].name());
    // println!("Record type: {:?}", questions[0].query_type());
    // println!("Record class: {:?}", questions[0].query_class());

    // Check if the domain name is in the cache
    let domain = questions[0].name().to_string();
    match cache.get(&domain) {
        Some(ip) => {
            println!("Domain name is in cache");
            println!("IP address for {}: {}", domain, ip);
            println!("Query took {} ms", (std::time::Instant::now() - start_timestamp).as_millis());
        }
        None => {
            // Forward the request to Cloudflare
            let ipv4address = init_recursive_request(&buffer[..number_of_bytes]).await;
            cache.insert(domain, ipv4address.expect("Error inserting ip address into cache"));
            println!("Query took {} ms", (std::time::Instant::now() - start_timestamp).as_millis());
        },
    }
}

// Parse the dns request
fn parse_dns_request(buffer: &[u8]) -> Result<Message, std::io::Error> {
    let mut decoder = BinDecoder::new(buffer);
    match Message::read(&mut decoder) {
        Ok(message) => Ok(message),
        Err(e) => Err(e.into()),
    }
}

// Initialize a recursive request
async fn init_recursive_request(buf: &[u8]) -> Result<Ipv4Addr, std::io::Error> {
    // Create socket
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    // Send query to forwarding server
    socket.send_to(buf, "1.1.1.3:53").unwrap();

    // Receive response
    let mut response = [0; 512];
    socket.recv_from(&mut response).unwrap();

    // Parse response
    let forwarded_response = parse_dns_request(&response).unwrap();

    println!("Raw response from Cloudflare: {:#?}", forwarded_response);

    let parsed_response = forwarded_response
        .answers()[0].clone()
        .data().expect("Error parsing response").clone();

    // parse rdata into A rocord, and get only ip address
    let a_record = parsed_response.clone()
        .into_a()
        .expect("Error parsing Cloudflare's response")
        .0;

    // Log response details
    println!("Response from Cloudflare: {:#?}", a_record);

    Ok(a_record)
}