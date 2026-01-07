# Custom Domain Setup with Cloudflare & Fly.io

Connect your Cloudflare-managed domain to your Fly.io deployment.

---

## Prerequisites

- Domain registered and managed in Cloudflare
- Fly.io CLI installed and authenticated
- App deployed to Fly.io (`fly deploy`)

---

## Quick Setup

### 1. Add Domain to Fly.io

```bash
# Add your domain
fly certs add yourdomain.com

# Verify it was added
fly certs list
```

### 2. Get Your App's IP Address

```bash
fly ips list
```

Example output:
```
VERSION  IP                      TYPE
v4       123.45.67.89            public
v6       2a09:8280:1::1:2345     public
```

### 3. Configure Cloudflare DNS

In your [Cloudflare Dashboard](https://dash.cloudflare.com/) → DNS → Records:

#### For Apex Domain (`yourdomain.com`)

| Type | Name | Content | Proxy |
|------|------|---------|-------|
| `A` | `@` | `123.45.67.89` | DNS only (grey cloud) |
| `AAAA` | `@` | `2a09:8280:1::1:2345` | DNS only (grey cloud) |

#### For Subdomain (`www.yourdomain.com`)

```bash
# Subdomains work automatically with fly certs
fly certs add www.yourdomain.com
```

### 4. Verify Configuration

```bash
fly certs check yourdomain.com
```

Expected output:
```
The certificate for yourdomain.com has been issued.
Hostname                  = yourdomain.com
DNS Provider              = cloudflare
Certificate Authority     = Let's Encrypt
Issued                    = ecdsa, rsa
```

---

## Important: Proxy Settings

⚠️ **For Fly.io to work correctly, disable Cloudflare's proxy (orange cloud):**

| Setting | Value |
|---------|-------|
| Proxy status | **DNS only** (grey cloud) |
| SSL/TLS mode | Full (strict) — if using Cloudflare proxy |

Fly.io handles SSL certificates automatically via Let's Encrypt.

---

## DNS Propagation

DNS changes typically take:
- **Minutes**: Most locations
- **Up to 48 hours**: Full global propagation

Check propagation:
```bash
# Check A record
dig yourdomain.com A +short

# Check if SSL works
curl -I https://yourdomain.com
```

---

## Troubleshooting

### Domain Not Resolving

```bash
# Verify DNS is set correctly
dig yourdomain.com A +short

# Should return your Fly.io IP
# If empty or wrong, check Cloudflare DNS records
```

### SSL Certificate Errors

```bash
# Check certificate status
fly certs show yourdomain.com

# Force certificate renewal
fly certs remove yourdomain.com
fly certs add yourdomain.com
```

### Mixed Content Warnings

The `fly.toml` includes `force_https = true`. Ensure all resources in your app use HTTPS URLs.

### Apex Domain Works, www Doesn't (or vice versa)

Add both domains:
```bash
fly certs add yourdomain.com
fly certs add www.yourdomain.com
```

---

## Optional: Redirect www to Apex

Add a Cloudflare Page Rule or Redirect Rule:

1. Go to **Rules** → **Redirect Rules**
2. Create rule:
   - **When**: Hostname equals `www.yourdomain.com`
   - **Then**: Redirect to `https://yourdomain.com` (301)

---

## Useful Commands

```bash
fly certs list              # List all certificates
fly certs show DOMAIN       # Show certificate details
fly certs check DOMAIN      # Verify DNS configuration
fly certs remove DOMAIN     # Remove a certificate
fly ips list                # Show app IP addresses
```

---

## Resources

- [Fly.io Custom Domains](https://fly.io/docs/networking/custom-domains/)
- [Cloudflare DNS](https://developers.cloudflare.com/dns/)
- [Fly.io TLS/SSL](https://fly.io/docs/networking/tls/)