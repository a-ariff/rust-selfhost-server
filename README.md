# rust-selfhost-server

A Rust Axum-based HTTP server with Docker Compose, Traefik reverse proxy, and Let's Encrypt HTTPS support.

## Features

- 🦀 **Rust-based** HTTP server using Axum framework
- 🐳 **Docker Compose** setup for easy deployment
- 🔒 **Traefik** reverse proxy with automatic HTTPS via Let's Encrypt
- 🔄 **GitHub Actions** CI/CD with Docker Build Cloud integration
- 📦 **Production-ready** configuration with security headers

## Quick Start

### Local Development

1. **Clone the repository:**
   ```bash
   git clone https://github.com/a-ariff/rust-selfhost-server.git
   cd rust-selfhost-server
   ```

2. **Set up environment variables:**
   ```bash
   cp .env.example .env
   # Edit .env with your domain name
   ```

3. **Configure your .env file:**
   ```env
   DOMAIN=your-domain.com
   TRAEFIK_ACME_EMAIL=your-email@example.com
   ```

4. **Run with Docker Compose:**
   ```bash
   docker compose up -d
   ```

### Production Deployment

#### Prerequisites

1. **VPS/Server** with Docker and Docker Compose installed
2. **Domain name** pointing to your server's IP address
3. **DNS records:**
   - A record: `your-domain.com` → `your-server-ip`
   - CNAME record: `www.your-domain.com` → `your-domain.com` (optional)

#### Setup Steps

1. **Clone the repository on your server:**
   ```bash
   git clone https://github.com/a-ariff/rust-selfhost-server.git
   cd rust-selfhost-server
   ```

2. **Create your .env file:**
   ```bash
   cp .env.example .env
   nano .env  # Edit with your configuration
   ```

   Example `.env` configuration:
   ```env
   DOMAIN=example.com
   TRAEFIK_ACME_EMAIL=admin@example.com
   ```

3. **Start the services:**
   ```bash
   docker compose up -d
   ```

4. **Verify deployment:**
   - Visit `https://your-domain.com` to see your Rust server
   - Visit `https://traefik.your-domain.com` to access Traefik dashboard (if enabled)

## Architecture

### Services

- **rust-server**: The main Rust application (port 3000)
- **traefik**: Reverse proxy handling HTTPS and routing

### Traefik Configuration

- **Automatic HTTPS**: Let's Encrypt certificates via HTTP-01 challenge
- **Security headers**: HSTS, X-Frame-Options, Content-Security-Policy
- **Rate limiting**: 100 requests/second with burst of 50
- **Compression**: Automatic gzip compression

## GitHub Actions Deployment

The repository includes automated deployment via GitHub Actions:

### Setup GitHub Secrets

Go to your repository → Settings → Secrets and Variables → Actions → Environment secrets.

Create a new environment called `production` and add these secrets:

- `DEPLOY_HOST`: Your VPS IP address or hostname
- `DEPLOY_USER`: SSH username for your VPS
- `DEPLOY_SSH_KEY`: Your private SSH key for server access

### Deployment Triggers

- **Tag push**: Push tags matching `v*` (e.g., `v1.0.0`)
- **Manual dispatch**: Trigger deployment manually from GitHub Actions tab

### Deployment Process

1. Creates a new tag: `git tag v1.0.0 && git push origin v1.0.0`
2. GitHub Actions connects to your VPS via SSH
3. Pulls latest changes: `docker compose pull`
4. Restarts services: `docker compose up -d`

## Configuration Files

### docker-compose.yml
Defines the multi-container setup with Traefik and the Rust server.

### traefik/traefik.yml
Main Traefik configuration with HTTPS setup and Let's Encrypt integration.

### traefik/dynamic.yml
Dynamic Traefik configuration with security middlewares and TLS settings.

### .env.example
Template for environment variables needed for deployment.

## Development

### Building locally
```bash
# Build the Rust application
cargo build --release

# Build Docker image
docker build -t rust-selfhost-server .

# Run locally
cargo run
```

### Testing
```bash
# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy
```

## Security

- **HTTPS-only**: All traffic redirected to HTTPS
- **Security headers**: Comprehensive security header configuration
- **Rate limiting**: Protection against abuse
- **Let's Encrypt**: Automatic certificate management
- **Docker security**: Non-root container execution

## Monitoring

- **Traefik Dashboard**: Available at `https://traefik.your-domain.com`
- **Access logs**: JSON-formatted access logs
- **Prometheus metrics**: Available for monitoring integration

## Troubleshooting

### Common Issues

1. **Certificate generation fails:**
   - Verify domain points to your server
   - Check port 80 is accessible for HTTP-01 challenge
   - Review Traefik logs: `docker compose logs traefik`

2. **Service not accessible:**
   - Verify .env configuration
   - Check service status: `docker compose ps`
   - Review logs: `docker compose logs`

3. **Deployment fails:**
   - Verify GitHub secrets are correctly set
   - Check SSH connectivity to your VPS
   - Review GitHub Actions logs

### Log Commands
```bash
# View all logs
docker compose logs

# View specific service logs
docker compose logs rust-server
docker compose logs traefik

# Follow logs in real-time
docker compose logs -f
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Support

If you encounter any issues or have questions:

1. Check the [Issues](https://github.com/a-ariff/rust-selfhost-server/issues) page
2. Create a new issue if your problem isn't already reported
3. Provide detailed information about your environment and the problem
