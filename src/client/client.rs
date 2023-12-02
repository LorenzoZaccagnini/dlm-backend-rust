use ecies::{encrypt, utils::generate_keypair};
use hex::ToHex;
use position_share::position_client::PositionClient;
use position_share::SendPositionRequest;
use rand::Rng;

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

    // ask user for public key
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


    // plain payload
    let payload = create_random_coordinate();

    // encrypt payload
    let encrypted_payload = &encrypt(&pk_target, payload.as_bytes()).unwrap();

    let mut client = PositionClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(SendPositionRequest {
        encpayload: encrypted_payload.encode_hex::<String>(),
        geo_sender_pubkey: public_key_string,
    });

    let response = client.send_position(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}

// create a function that creates a random latitude and longitude
// it should return two strings
fn create_random_coordinate() -> String {
    // array of possible x coordinates

    let lats = vec![
        "-90", "-89", "-88", "-87", "-86", "-85", "-84", "-83", "-82",
    ];

    // array of possible longitude coordinates

    let lons = vec![
        "-180", "-179", "-178", "-177", "-176", "-175", "-174", "-173", "-172", "-171", "-170",
        "-169", "-168", "-167", "-166", "-165", "-164",
    ];

    // generate random number between 0 and 8

    let lat_index = rand::thread_rng().gen_range(0..8);

    // generate random number between 0 and 16

    let lon_index = rand::thread_rng().gen_range(0..16);

    // return the random latitude and longitude in one string

    return format!("{}:{}", lats[lat_index], lons[lon_index]);
}
