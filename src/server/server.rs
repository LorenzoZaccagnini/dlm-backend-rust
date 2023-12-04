use std::collections::HashMap;
use std::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};
use position_share::position_server::{Position, PositionServer};
use position_share::{SendPositionRequest, SendPositionResponse, GetPositionRequest, GetPositionResponse, CloseSessionRequest, CloseSessionResponse};
use k256::ecdsa::{ Signature, VerifyingKey, signature::Verifier};

pub mod position_share {
    tonic::include_proto!("positionshare");
}

pub struct Payload {
    encpayload: String,
    geo_sender_pubkey: String,
}

#[derive(Default)]
pub struct MyPosition {
    payloads: Mutex<HashMap<String, Payload>>,
}

#[tonic::async_trait]
impl Position for MyPosition {

    async fn send_position(
        &self,
        request: Request<SendPositionRequest>,
    ) -> Result<Response<SendPositionResponse>, Status> {
        println!("Got a request: {:?}", request);

        let request = request.into_inner();
        let encpayload = request.encpayload;
        let geo_sender_pubkey = &request.geo_sender_pubkey;

        let payload = Payload {
            encpayload: encpayload,
            geo_sender_pubkey: geo_sender_pubkey.clone(),
        };

        let mut payloads = self.payloads.lock().unwrap();
        payloads.insert(geo_sender_pubkey.to_string(), payload);

        let reply = position_share::SendPositionResponse {
            success: true,
        };

        Ok(Response::new(reply))
    }

    async fn get_position(
        &self,
        request: Request<GetPositionRequest>,
    ) -> Result<Response<GetPositionResponse>, Status> {
        println!("Got a request: {:?}", request);

        // get public key from request
        let geo_sender_pubkey = request.into_inner().geo_sender_pubkey;

        // get payloads from hashmap
        let payloads = self.payloads.lock().unwrap();

        // get payload from hashmap
        let payload = payloads.get(&geo_sender_pubkey);

        let reply = position_share::GetPositionResponse {
            encpayload: payload.unwrap().encpayload.clone(),
        };

        Ok(Response::new(reply))
    }

    async fn close_session(
        &self,
        request: Request<CloseSessionRequest>,
    ) -> Result<Response<CloseSessionResponse>, Status> {
        println!("Got a request: {:?}", request);

        let (geo_sender_pubkey, signature, close_session_msg) = {
            let inner = request.into_inner();
            (inner.geo_sender_pubkey, inner.signature, inner.close_session_msg)
        };

        // decode message
        let close_session_msg = hex::decode(close_session_msg).unwrap();
        let close_session_msg = String::from_utf8(close_session_msg).unwrap();

        // signature
        let signature = hex::decode(signature).unwrap();
        let signature: Signature = Signature::from_slice(&signature).unwrap();

        // decode public key
        let geo_sender_pubkey_clone = geo_sender_pubkey.clone();
        let geo_sender_pubkey = hex::decode(geo_sender_pubkey).unwrap();
        let public_key_sec = geo_sender_pubkey.as_slice();

        let verify_key_result = VerifyingKey::from_sec1_bytes(public_key_sec);
    

        let verification_success = if let Ok(verify_key) = verify_key_result {
            let rtn = verify_key.verify(close_session_msg.as_bytes(), &signature).is_ok();
            println!("Verification result: {:?}", rtn);
            rtn
        } else {
            println!("Failed to create verifying key");
            false
        };

        let mut payloads = self.payloads.lock().unwrap();
        
        // delete session from hashmap using public key as index
        if verification_success {
            match payloads.remove(&geo_sender_pubkey_clone) {
                Some(_) => println!("Session deleted"),
                None => println!("Session not found"),
            }
        }

        // if false exit all
        if !verification_success {
            println!("Verification failed");
            return Err(Status::invalid_argument("Verification failed"));
        }
        
        let reply = position_share::CloseSessionResponse {
            success: verification_success,
        };
        
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let position = MyPosition::default();

    println!("Dammi la mano server listening on {}", addr);

    Server::builder()
        .add_service(PositionServer::new(position))
        .serve(addr)
        .await?;

    Ok(())
}