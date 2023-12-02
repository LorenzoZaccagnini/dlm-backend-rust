use ecies::{encrypt, decrypt, utils::generate_keypair};
use hex::ToHex;
use position_share::position_client::PositionClient;
use position_share::GetPositionRequest;

pub mod position_share {
    tonic::include_proto!("positionshare");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sk, pk) = generate_keypair();
    #[cfg(not(feature = "x25519"))]
    let (sk, pk) = (&sk.serialize(), &pk.serialize());
    #[cfg(feature = "x25519")]
    let (sk, pk) = (sk.as_bytes(), pk.as_bytes());

    // 1. print public key

    println!("Public key to pass: {}", pk.encode_hex::<String>());
    
    // 2. ask for target public key
    use std::io::{stdin,stdout,Write};
    let mut s=String::new();
    print!("Enter listener public key: ");
    let _=stdout().flush();
    stdin().read_line(&mut s).expect("Did not enter a correct string");
    if let Some('\n')=s.chars().next_back() {
        s.pop();
    }
    if let Some('\r')=s.chars().next_back() {
        s.pop();
    }

    println!("Public key: {}", s);

    // convert public key from string to bytes
    let pk_target = hex::decode(s).unwrap();

    // convert public key to string
    let public_key_string = pk.encode_hex::<String>();

    // call get position
    let mut client = PositionClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(GetPositionRequest {
        geo_sender_pubkey: public_key_string,
    });

    let response = client.get_position(request).await?;

    println!("RESPONSE={:?}", response);

    // decrypt payload
    let decrypted_payload = decrypt(sk, &hex::decode(response.into_inner().encpayload).unwrap()).unwrap();

    println!("Decrypted payload: {:?}", decrypted_payload);

    Ok(())

}
