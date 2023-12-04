# Dammi la mano backend
Dammi la mano is an e2e application made for sharing the user position with another user. Dammi la mano was developed by me and Elisa Romondia, with this project we won the TIM WCAP prize a the first Italian official hackathon, Digithon 2016, the version in this repo is a Rewrite in Rust version. Dammi la mano uses GRPC for communication and stores the data in the app memory. The data is encrypted with ECIES secp256k1 and Xchacha20poly1305. The data is stored in the app memory and is deleted when the session is closed. The session is closed when the user signs a message with his private key and the backend checks if the signature is valid. 

## Original presentation video

[Dammi la mano video](https://www.youtube.com/watch?v=abycMMQ_CG0)

## How to run

Run server
```bash
cargo run --bin server
```

Run client
```bash
cargo run --bin client
```

The server will listen on port 50051 and the client will connect to localhost:50051, the client will simulate two users, Alice and Bob. Alice will send her position to Bob and Bob will decrypt the message and see Alice's position. At the end of the session, Alice will close the session and the message will be deleted from the server.

## How E2E is implemented

Alice wants to share her GPS position with Bob without leaving Eve able to see it. Dammi la mano enables Alice to encrypt her position with Bob's public key and send it to Bob. Bob will be able to decrypt the message with his private key and see Alice's position.

1. Alice generates a key pair (private and public key), the curve used is secp256k1.
2. Bob generates a key pair (private and public key)
3. Alice encrypts her position with Bob's public key, in this case, ECC and Xchacha20poly1305 are used inside an ECIES algorithm.
4. Alice sends the encrypted message to Dammi la mano backend because Alice and Bob can't communicate directly with their public IP address.
5. Dammi la mano backend receives the encrypted message and stores it into a hashmap indexed by Alice's public key.
6. Bob asks Dammi la mano backend for Alice's position.
7. Dammi la mano backend sends the encrypted message to Bob.
8. Bob decrypts the message with his private key and sees Alice's position.
9. Alice closes the session and Dammi la mano backend deletes the session from the memory. When Alice wants to close the session signs a message with her private key and Dammi la mano backend checks if the signature is valid, if it is the session is closed and the session is deleted.
10. TODO Optionally a cronjob can be set to delete an expired session.