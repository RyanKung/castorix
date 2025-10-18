# REST API Security Policy

## 🔒 Security Design

The Castorix REST API Server is designed as a **READ-ONLY** interface that **NEVER** touches private keys or performs signing operations.

### Security Guarantees

✅ **No Private Key Access**
- The API server does NOT have access to any private keys
- The API server does NOT have access to encrypted key storage
- The API server does NOT have access to custody wallets
- The API server does NOT have access to Ed25519 signer keys

✅ **Read-Only Operations**
- All endpoints use HTTP `GET` method (no POST/PUT/DELETE)
- All operations are queries only
- No message signing
- No transaction broadcasting
- No proof generation
- No signer registration
- No FID registration

✅ **No Authentication Required**
- The API server does NOT require authentication
- This is intentional - all data queried is public on the Farcaster network
- Private operations are NOT exposed

### Architecture

```
REST API Server (Port 3000)
├── FarcasterClient (READ-ONLY mode, key_manager = None)
├── FarcasterContractClient (Query-only operations)
└── ENS Resolution (Query-only operations)
```

**Key Point**: The `FarcasterClient` is initialized with `None` for the key manager:
```rust
let hub_client = Arc::new(FarcasterClient::new(self.hub_url.clone(), None));
//                                                                     ^^^^ No key manager
```

### What the API CAN Do

- ✅ Query user information from Farcaster Hub
- ✅ Check spam status
- ✅ Query contract state (FID prices, storage prices)
- ✅ Resolve ENS domains (read-only)
- ✅ Check if an address has a FID

### What the API CANNOT Do

- ❌ Sign messages
- ❌ Submit casts
- ❌ Register FIDs
- ❌ Register signers
- ❌ Rent storage
- ❌ Generate username proofs
- ❌ Access or export private keys
- ❌ Decrypt encrypted keys
- ❌ Access custody wallets
- ❌ Perform any transaction that requires signatures

## 🛡️ Security Best Practices

### For Deployment

1. **Network Security**
   - Deploy behind a firewall
   - Use HTTPS in production (via reverse proxy)
   - Limit access to trusted networks if needed

2. **Rate Limiting**
   - Implement rate limiting to prevent abuse
   - Recommended: 100 requests/minute per IP

3. **Monitoring**
   - Monitor for unusual access patterns
   - Log all requests for audit purposes
   - Set up alerts for high traffic

4. **CORS Configuration**
   - Current: Permissive (all origins allowed)
   - Production: Restrict to your domain(s)
   - Example: `.allow_origin("https://yourapp.com")`

### Environment Variables

The API server only needs public endpoints:
```bash
# Required
FARCASTER_HUB_URL=https://hub-api.neynar.com

# Optional (for additional query features)
ETH_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
ETH_BASE_RPC_URL=https://mainnet.base.org
ETH_OP_RPC_URL=https://mainnet.optimism.io
```

**Important**: 
- ❌ DO NOT set `PRIVATE_KEY` environment variable
- ❌ DO NOT run the API server with access to `~/.castorix/keys/`
- ❌ DO NOT run the API server with access to `~/.castorix/custody/`
- ❌ DO NOT run the API server with access to `~/.castorix/ed25519/`

## 🚨 Security Incident Response

If you discover a security vulnerability:

1. **DO NOT** open a public issue
2. Email security concerns to the maintainers
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

## 🔐 Comparison with CLI

| Operation | CLI | REST API |
|-----------|-----|----------|
| Query user data | ✅ | ✅ |
| Check spam | ✅ | ✅ |
| Query contracts | ✅ | ✅ |
| Generate keys | ✅ | ❌ |
| Sign messages | ✅ | ❌ |
| Submit casts | ✅ | ❌ |
| Register FID | ✅ | ❌ |
| Register signers | ✅ | ❌ |
| Create proofs | ✅ | ❌ |

## 📝 Audit Log

### Security Reviews
- 2025-01-18: Initial security review - No private key exposure confirmed
- Future: Regular security audits recommended

### Known Limitations
- The API is currently in early development
- Some endpoints are placeholders (not yet implemented)
- Production hardening is recommended before public deployment

## ✅ Security Checklist for Developers

Before adding new endpoints:

- [ ] Does this endpoint require signing? → If YES, DO NOT add to REST API
- [ ] Does this endpoint access private keys? → If YES, DO NOT add to REST API
- [ ] Does this endpoint modify blockchain state? → If YES, DO NOT add to REST API
- [ ] Is this endpoint query-only? → If YES, safe to add
- [ ] Does this endpoint query public data? → If YES, safe to add

## 🎯 Design Principle

**The REST API Server is designed to be safely exposed to the internet without risking private key compromise.**

All sensitive operations (key generation, signing, transaction broadcasting) are ONLY available through the CLI tool, which runs locally and requires explicit user interaction.

---

Last Updated: 2025-01-18
Version: 1.0

