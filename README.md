# sails-program-verifier

`sails-program-verifier` is a service for verifying programs and smart contracts written in [Sails](https://github.com/gear-tech/sails).
It ensures that a compiled program meets the required versions of Sails, Rust, and the operating system.

## How It Works

To verify a program, it must be compiled with specific versions of Sails, Rust, and other dependencies.

To ensure consistency, the repository includes Docker images that can be used for program compilation.

The verification service itself relies on these images when processing verification requests.

## Compiling a Contract

Developers can use the provided Docker images to compile their programs with the correct environment. Here’s an example command:

```sh
docker run -v $(pwd):/app --entrypoint /bin/bash ghcr.io/gear-tech/sails-program-builder:<version> -c 'cargo build --release'
```

Check available versions [here](https://github.com/gear-tech/sails-program-verifier/pkgs/container/sails-program-builder)

## API Documentation

The `sails-program-verifier` service provides a REST API for verifying Sails programs. Below are the available endpoints.


### 1. Get Verified Code
**Endpoint:** `GET /code`
**Description:** Retrieves verified code by its ID.

**Query Parameters:**
- `id` *(string, required)* – The unique identifier of the verified code.

**Response:**
```json
{
  "id": "12345",
  "idl_hash": "abcdef123456",
  "name": "MyContract",
  "repo_link": "https://github.com/user/repo"
}
```

---

### 2. Get IDL for Verified Code
**Endpoint:** `GET /idl`
**Description:** Retrieves the IDL (Interface Definition Language) for a verified contract by ID.

**Query Parameters:**
- `id` *(string, required)* – The unique identifier of the verified code.

**Response:**
```json
{
  "id": "12345",
  "content": "IDL data here..."
}
```

---

### 3. Get Supported Sails Versions
**Endpoint:** `GET /supported_versions`
**Description:** Returns a list of Sails versions supported by the verifier.

**Response:**
```json
[
  "0.7.1",
  "0.7.2",
  "0.8.0"
]
```

---

### 4. Submit a Verification Request
**Endpoint:** `POST /verify`
**Description:** Submits a program for verification.

**Request Body:**
```json
{
  "repo_link": "https://github.com/user/repo",
  "version": "0.7.1",
  "network": "testnet",
  "code_id": "12345",
  "build_idl": true
}
```

**Response:**
```json
{
  "id": "verification-request-id"
}
```

---

### 5. Check Verification Status
**Endpoint:** `GET /verify/status`
**Description:** Checks the status of a verification request.

**Query Parameters:**
- `id` *(string, required)* – The ID of the verification request.

**Response:**
```json
{
  "status": "completed",
  "created_at": 1700000000,
  "failed_reason": null
}
```
Possible `status` values:
- `"pending"` – Verification is in progress.
- `"completed"` – Verification was successful.
- `"failed"` – Verification failed (see `failed_reason` for details).

---
