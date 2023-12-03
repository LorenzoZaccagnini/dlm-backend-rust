use ecies::decrypt;
use ecies::{encrypt, utils::generate_keypair};
use hex::ToHex;
use position_share::position_client::PositionClient;
use position_share::{GetPositionRequest, SendPositionRequest, SendPositionResponse, GetPositionResponse, CloseSessionRequest, CloseSessionResponse};
use rand::Rng;

pub mod position_share {
    tonic::include_proto!("positionshare");
}

async fn send_position(pk_receiver: &[u8; 65], pk_sender: &[u8; 65]) -> Result<tonic::Response<SendPositionResponse>, Box<dyn std::error::Error>> {
    // convert receiver public key to str, just for simulation that it is received from the other party
    let pk_receiver_string = pk_receiver.encode_hex::<String>();

    // convert receiver public key from string to bytes
    let pk_receiver_decoded = hex::decode(pk_receiver_string).unwrap();

    // convert sender public key to string
    let public_key_sender_string = pk_sender.encode_hex::<String>();

    // plain payload
    let payload = create_random_coordinate();

    // encrypt payload
    let encrypted_payload = &encrypt(&pk_receiver_decoded, payload.as_bytes()).unwrap();

    let request = tonic::Request::new(SendPositionRequest {
        encpayload: encrypted_payload.encode_hex::<String>(),
        geo_sender_pubkey: public_key_sender_string.clone(),
    });

    let mut client = PositionClient::connect("http://[::1]:50051").await?;

    let response = client.send_position(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(response)
}

async fn get_position(pk_sender: &[u8; 65], sk_receiver: &[u8; 32]) -> Result<tonic::Response<GetPositionResponse>, Box<dyn std::error::Error>> {
    let rec_request = tonic::Request::new(GetPositionRequest {
        geo_sender_pubkey: pk_sender.encode_hex::<String>()
    });

    let mut client = PositionClient::connect("http://[::1]:50051").await?;

    let response = client.get_position(rec_request).await?;

    println!("RESPONSE RECEIVER={:?}", response);

    // convert encrypted payload from string to bytes
    let inner_response = response.into_inner();

    let enc_rec_payload = inner_response.encpayload.clone();

    // decode hex
    let enc_rec_payload = hex::decode(enc_rec_payload).unwrap();

    // decrypt payload
    let decrypted_rec_payload = &decrypt(sk_receiver, enc_rec_payload.as_slice()).unwrap();

    // convert decrypted payload from bytes to string
    let decrypted_rec_payload_string = String::from_utf8(decrypted_rec_payload.to_vec()).unwrap();

    println!("DECRYPTED PAYLOAD={:?}", decrypted_rec_payload_string);

    Ok(tonic::Response::new(inner_response))
}

async fn close_session(pk_sender: &[u8; 65]) -> Result<tonic::Response<CloseSessionResponse>, Box<dyn std::error::Error>> {

    const MGS : &str = "close session";


    let request = tonic::Request::new(CloseSessionRequest {
        geo_sender_pubkey: pk_sender.encode_hex::<String>(),
        signed_close_session_msg: MGS.to_string(),
    });

    let mut client = PositionClient::connect("http://[::1]:50051").await?;

    let response = client.close_session(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // generate sender keypair
    let (sk_sender, pk_sender) = generate_keypair();
    #[cfg(not(feature = "x25519"))]
    let (sk_sender, pk_sender) = (&sk_sender.serialize(), &pk_sender.serialize());
    #[cfg(feature = "x25519")]
    let (sk_sender, pk_sender) = (sk_sender.as_bytes(), pk_sender.as_bytes());

    // print sender keys
    println!("Sender public key: {}", pk_sender.encode_hex::<String>());
    println!("Sender private key: {}", sk_sender.encode_hex::<String>());

    // generate receiver keypair
    let (sk_receiver, pk_receiver) = generate_keypair();
    #[cfg(not(feature = "x25519"))]
    let (sk_receiver, pk_receiver) = (&sk_receiver.serialize(), &pk_receiver.serialize());
    #[cfg(feature = "x25519")]
    let (sk_receiver, pk_receiver) = (sk_receiver.as_bytes(), pk_receiver.as_bytes());

    // print receiver keys
    println!("Receiver public key: {}", pk_receiver.encode_hex::<String>());
    println!("Receiver private key: {}", sk_receiver.encode_hex::<String>());

    // simulate sending and receiving of coordinates
    for _ in 0..4 {
        send_position(pk_receiver, pk_sender).await?;
        get_position(pk_sender, sk_receiver).await?;
    }

    // close session
    close_session(pk_sender).await?;

    Ok(())
}

// create a function that creates a random latitude and longitude
fn create_random_coordinate() -> String {
    let lat_lng = vec!["41.8931:12.4828", "40.7128:74.0060", "51.5074:0.1278", "48.8566:2.3522", "55.7558:37.6173", "52.5200:13.4050", "37.7749:122.4194", "35.6895:139.6917", "37.5665:126.9780", "19.4326:-99.1332", "35.6762:139.6503", "41.0082:28.9784", "35.6892:51.3890", "31.2304:121.4737", "31.2244:121.4759"];
    let lat_lng_index = rand::thread_rng().gen_range(0..15);

    return lat_lng[lat_lng_index].to_string();
}
