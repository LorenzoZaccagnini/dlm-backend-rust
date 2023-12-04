# Dammi la mano backend
Dammi la mano is an e2e application made for sharing the user position with another user. Dammi la mano developed by me and Elisa Romondia won the TIM WCAP prize a the first italian official hackathon, Digithon 2017, the version in this repo is a RIR. Dammi la mano uses GRPC for communication and stores the data in the app memory. The data is encrypted with ECIES secp256k1 and Xchacha20poly1305.


## How E2E is implemented

Alice wants to share her gps position with Bob without leaving Eve able to see it. Dammi la mano enables Alice to encrypt her position with Bob's public key and send it to Bob. Bob will be able to decrypt the message with his private key and see Alice's position.

1. Alice generates a key pair (private and public key), the curve used is secp256k1.
2. Bob generates a key pair (private and public key)
3. Alice encrypts her position with Bob's public key, in this case ECC and Xchacha20poly1305 are used inside an ECIES algorithm.
4. Alice sends the encrypted message to Dammi la mano backend, because Alice and Bob can't communicate directly with their public IP address.
5. Dammi la mano backend receives the encrypted message, add an expiration time and store it in a database
6. Bob asks Dammi la mano backend for Alice's position.
7. Dammi la mano backend checks if the message is expired, if not it sends the encrypted message to Bob. If the message is expired it deletes it from the database.
8. Bob decrypts the message with his private key and sees Alice's position.
9. Alice closes the session and Dammi la mano backend deletes the message from the database. When Alice wants to close the session signs a message with her private key and Dammi la mano backend checks if the signature is valid, if it is the session is closed and the message is deleted from the database.
10. Optionally a cronjob can be set to delete expired messages from the database.

