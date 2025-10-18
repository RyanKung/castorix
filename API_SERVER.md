# Castorix REST API Server

## Overview

The Castorix REST API Server provides a traditional HTTP RESTful API for querying Farcaster protocol data. This is an alternative to the MCP (Model Context Protocol) server, designed for standard web and application integrations.

## 🔒 Security Notice

**IMPORTANT**: This API server is designed as a **READ-ONLY** interface that **NEVER** touches private keys.

- ✅ Safe to expose to the internet
- ✅ Only performs query operations
- ✅ No private key access
- ✅ No signing operations
- ✅ No transaction broadcasting

All sensitive operations (key generation, signing, transactions) are ONLY available through the CLI tool.

For detailed security information, see [SECURITY_API.md](SECURITY_API.md).

## Starting the Server

```bash
# Start on default port (3000)
castorix api serve

# Start on custom port
castorix api serve --port 8080

# Start on specific host
castorix api serve --host 127.0.0.1 --port 3000
```

## Environment Variables

```bash
# Required
FARCASTER_HUB_URL=https://hub-api.neynar.com

# Optional - for ENS endpoints
ETH_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
ETH_BASE_RPC_URL=https://mainnet.base.org

# Optional - for contract endpoints
ETH_OP_RPC_URL=https://mainnet.optimism.io
```

## API Endpoints

### Health Check

**GET** `/health`

Check if the API server is running.

```bash
curl http://localhost:3000/health
```

**Response:**
```json
{
  "status": "ok",
  "service": "castorix-api",
  "version": "0.1.0"
}
```

---

### Hub Endpoints

#### Get User Information

**GET** `/api/hub/users/:fid`

Get basic user information by FID.

```bash
curl http://localhost:3000/api/hub/users/3
```

**Response:**
```json
{
  "success": true,
  "data": {
    "fid": 3,
    "username": "dwr",
    "displayName": "Dan Romero",
    // ... more user data
  }
}
```

#### Get User Profile

**GET** `/api/hub/users/:fid/profile`

Get detailed user profile (currently returns same as user info).

```bash
curl http://localhost:3000/api/hub/users/3/profile
```

#### Get User Statistics

**GET** `/api/hub/users/:fid/stats`

Get user statistics including followers and following counts.

```bash
curl http://localhost:3000/api/hub/users/3/stats
```

#### Get Followers

**GET** `/api/hub/users/:fid/followers?limit=100`

Get list of followers (currently not implemented - placeholder).

```bash
curl http://localhost:3000/api/hub/users/3/followers?limit=50
```

#### Get Following

**GET** `/api/hub/users/:fid/following?limit=100`

Get list of users that this FID follows (currently not implemented - placeholder).

```bash
curl http://localhost:3000/api/hub/users/3/following?limit=50
```

#### Get Ethereum Addresses

**GET** `/api/hub/users/:fid/addresses`

Get verified Ethereum addresses (currently not implemented - placeholder).

```bash
curl http://localhost:3000/api/hub/users/3/addresses
```

#### Get ENS Domains

**GET** `/api/hub/users/:fid/ens`

Get ENS domains for a FID (currently not implemented - placeholder).

```bash
curl http://localhost:3000/api/hub/users/3/ens
```

#### Get Custody Address

**GET** `/api/hub/users/:fid/custody`

Get custody address for a FID (currently not implemented - placeholder).

```bash
curl http://localhost:3000/api/hub/users/3/custody
```

#### Get User Casts

**GET** `/api/hub/users/:fid/casts?limit=100`

Get recent casts by FID (currently not implemented - placeholder).

```bash
curl http://localhost:3000/api/hub/users/460432/casts?limit=10
```

#### Check Spam Status

**GET** `/api/hub/spam/:fid`

Check if a FID is marked as spam.

```bash
curl http://localhost:3000/api/hub/spam/12345
```

**Response:**
```json
{
  "success": true,
  "data": {
    "fid": 12345,
    "is_spam": false
  }
}
```

#### Get Hub Info

**GET** `/api/hub/info`

Get Hub information and sync status (currently not implemented - placeholder).

```bash
curl http://localhost:3000/api/hub/info
```

---

### ENS Endpoints

> **Note**: ENS endpoints require `ETH_RPC_URL` and `ETH_BASE_RPC_URL` environment variables.

#### Resolve ENS Domain

**GET** `/api/ens/resolve/:domain`

Resolve an ENS domain to an Ethereum address (currently not implemented - placeholder).

```bash
curl http://localhost:3000/api/ens/resolve/vitalik.eth
```

#### Verify Domain Ownership

**GET** `/api/ens/verify/:domain/:address`

Verify if an address owns a specific ENS domain (currently not implemented - placeholder).

```bash
curl http://localhost:3000/api/ens/verify/vitalik.eth/0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
```

---

### Contract Endpoints

> **Note**: Contract endpoints require `ETH_OP_RPC_URL` environment variable.

#### Get FID Registration Price

**GET** `/api/contract/fid/price`

Get the current price to register a new FID.

```bash
curl http://localhost:3000/api/contract/fid/price
```

**Response:**
```json
{
  "success": true,
  "data": {
    "price_wei": "1000000000000000",
    "price_eth": "0.001000"
  }
}
```

#### Get Storage Rental Price

**GET** `/api/contract/storage/price/:units`

Get the price to rent storage units.

```bash
curl http://localhost:3000/api/contract/storage/price/1
```

**Response:**
```json
{
  "success": true,
  "data": {
    "units": 1,
    "price_wei": "500000000000000",
    "price_eth": "0.000500"
  }
}
```

#### Check Address FID

**GET** `/api/contract/address/:address/fid`

Check if an Ethereum address has a registered FID.

```bash
curl http://localhost:3000/api/contract/address/0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045/fid
```

**Response:**
```json
{
  "success": true,
  "data": {
    "address": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
    "has_fid": true,
    "fid": 3
  }
}
```

---

## Error Responses

All endpoints return errors in a consistent format:

```json
{
  "success": false,
  "error": "Error message here"
}
```

**HTTP Status Codes:**
- `200 OK` - Request successful
- `400 Bad Request` - Invalid request parameters
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server-side error

## CORS

The API server is configured with permissive CORS settings for development:
- All origins allowed (`*`)
- All methods allowed
- All headers allowed

## Example Usage with JavaScript

```javascript
// Fetch user information
fetch('http://localhost:3000/api/hub/users/3')
  .then(res => res.json())
  .then(data => console.log(data));

// Check spam status
fetch('http://localhost:3000/api/hub/spam/12345')
  .then(res => res.json())
  .then(data => console.log(`Spam: ${data.data.is_spam}`));

// Get FID price
fetch('http://localhost:3000/api/contract/fid/price')
  .then(res => res.json())
  .then(data => console.log(`Price: ${data.data.price_eth} ETH`));
```

## Example Usage with Python

```python
import requests

# Get user information
response = requests.get('http://localhost:3000/api/hub/users/3')
user_data = response.json()
print(user_data)

# Check spam status
response = requests.get('http://localhost:3000/api/hub/spam/12345')
spam_data = response.json()
print(f"Spam: {spam_data['data']['is_spam']}")

# Get FID price
response = requests.get('http://localhost:3000/api/contract/fid/price')
price_data = response.json()
print(f"Price: {price_data['data']['price_eth']} ETH")
```

## Development Notes

### Current Status

This is an initial implementation with the following status:
- ✅ **Core infrastructure**: Working (server, routing, CORS, error handling)
- ✅ **Health check**: Working
- ✅ **Basic user query**: Working (`/api/hub/users/:fid`)
- ✅ **Spam check**: Working (`/api/hub/spam/:fid`)
- ✅ **Contract queries**: Working (FID price, storage price, address lookup)
- ⚠️ **Other Hub endpoints**: Placeholder (not fully implemented yet)
- ⚠️ **ENS endpoints**: Placeholder (not implemented yet)

### Future Enhancements

- Implement remaining Hub endpoints using Hub API
- Implement ENS resolution endpoints
- Add rate limiting
- Add authentication/API keys
- Add request logging and metrics
- Add caching layer for frequently accessed data
- Add WebSocket support for real-time updates

## Production Deployment

For production deployment, consider:

1. **Security**: Add authentication/authorization
2. **Rate Limiting**: Implement rate limiting to prevent abuse
3. **Load Balancing**: Use a load balancer for multiple instances
4. **HTTPS**: Deploy behind HTTPS proxy (nginx, Caddy)
5. **Environment**: Set proper environment variables
6. **Logging**: Configure structured logging
7. **Monitoring**: Set up health checks and metrics

### Example with Nginx

```nginx
server {
    listen 80;
    server_name api.example.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## Comparison with MCP Server

| Feature | REST API | MCP Server |
|---------|----------|------------|
| Protocol | HTTP/REST | JSON-RPC 2.0 over stdio |
| Use Case | Web apps, mobile apps | AI assistants (Claude Desktop) |
| Access | Network (HTTP) | Direct process communication |
| Integration | Any HTTP client | MCP-compatible clients |
| Authentication | Can add API keys | Process-level isolation |

## Support

For issues, questions, or contributions, please visit:
- GitHub: https://github.com/RyanKung/castorix
- Issues: https://github.com/RyanKung/castorix/issues

