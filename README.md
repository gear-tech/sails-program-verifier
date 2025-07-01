# sails-program-verifier

`sails-program-verifier` is a service for verifying programs written in [Sails](https://github.com/gear-tech/sails).
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

### Troubleshooting Docker Access Issues

If you encounter a `403 Forbidden` error when pulling the Docker image, particularly on Mac with Silicon chips, you need to authenticate with GitHub Container Registry:

1. Create a [personal access token (PAT)](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens) in GitHub:
   - Go to GitHub → Settings → Developer settings → Personal access tokens → Generate new token
   - Select the `read:packages` scope
   - Generate and copy your token

2. Log in to GitHub Container Registry using your token:
   ```sh
   echo YOUR_GITHUB_TOKEN | docker login ghcr.io -u YOUR_GITHUB_USERNAME --password-stdin
   ```
   Or:
   ```sh
   docker login ghcr.io -u YOUR_GITHUB_USERNAME -p YOUR_GITHUB_TOKEN
   ```

3. Try pulling the image again:
   ```sh
   docker pull ghcr.io/gear-tech/sails-program-builder:<version>
   ```

After successful authentication, you can run the Docker command from the previous section.

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
  "version": "0.8.0",
  "network": "testnet",
  "code_id": "0x12345",
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
