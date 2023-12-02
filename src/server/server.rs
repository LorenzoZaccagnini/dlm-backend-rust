use std::collections::HashMap;
use std::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

use position_share::position_server::{Position, PositionServer};
use position_share::{SendPositionRequest, SendPositionResponse, GetPositionRequest, GetPositionResponse, CloseSessionRequest, CloseSessionResponse};

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

        let reply = position_share::CloseSessionResponse {
            success: true,
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let position = MyPosition::default();

    println!("Position server listening on {}", addr);

    Server::builder()
        .add_service(PositionServer::new(position))
        .serve(addr)
        .await?;

    Ok(())
}