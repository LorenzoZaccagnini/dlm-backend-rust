use ecies::decrypt;
use ecies::{encrypt, utils::generate_keypair};
use hex::ToHex;
use position_share::position_client::PositionClient;
use position_share::{GetPositionRequest, SendPositionRequest};
use rand::Rng;

pub mod position_share {
    tonic::include_proto!("positionshare");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // generate sender keypair
    let (sk, pk) = generate_keypair();
    #[cfg(not(feature = "x25519"))]
    let (sk, pk) = (&sk.serialize(), &pk.serialize());
    #[cfg(feature = "x25519")]
    let (sk, pk) = (sk.as_bytes(), pk.as_bytes());

    // generate receiver keypair
    let (sk_target, pk_target) = generate_keypair();
    #[cfg(not(feature = "x25519"))]
    let (sk_target, pk_target) = (&sk_target.serialize(), &pk_target.serialize());
    #[cfg(feature = "x25519")]
    let (sk_target, pk_target) = (sk_target.as_bytes(), pk_target.as_bytes());

    // convert receiver public key to str, just for simulation that it is received from the other party
    let s = pk_target.encode_hex::<String>();

    // convert receiver public key from string to bytes
    let pk_target_decoded = hex::decode(s).unwrap();

    // convert sender public key to string
    let public_key_string = pk.encode_hex::<String>();

    // plain payload
    let payload = create_random_coordinate();

    // encrypt payload
    let encrypted_payload = &encrypt(&pk_target_decoded, payload.as_bytes()).unwrap();

    let mut client = PositionClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(SendPositionRequest {
        encpayload: encrypted_payload.encode_hex::<String>(),
        geo_sender_pubkey: public_key_string.clone(),
    });

    let response = client.send_position(request).await?;

    println!("RESPONSE={:?}", response);

    // simulate receiving a message
    let rec_request = tonic::Request::new(GetPositionRequest {
        geo_sender_pubkey: public_key_string
    });

    let rec_response = client.get_position(rec_request).await?;

    println!("RESPONSE RECEIVER={:?}", rec_response);

    // convert encrypted payload from string to bytes
    let enc_rec_payload = rec_response.into_inner().encpayload.clone();

    // decode hex
    let enc_rec_payload = hex::decode(enc_rec_payload).unwrap();

    // decrypt payload
    let decrypted_rec_payload = &decrypt(sk_target, enc_rec_payload.as_slice()).unwrap();

    // convert decrypted payload from bytes to string
    let decrypted_rec_payload_string = String::from_utf8(decrypted_rec_payload.to_vec()).unwrap();

    // decrypt(sk, &encrypt(pk, msg).unwrap()).unwrap().as_slice()

    println!("DECRYPTED PAYLOAD={:?}", decrypted_rec_payload_string);

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
