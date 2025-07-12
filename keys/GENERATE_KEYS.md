# Generate Keys Commands

Commands to generate public and private keys for user authentication (Linux).

---
### Generate private key
```bash
openssl genpkey -algorithm RSA -out private_key.pem -pkeyopt rsa_keygen_bits:2048
```

### Generate public key
```bash
openssl rsa -pubout -in private_key.pem -out public_key.pem
```