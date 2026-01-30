# Docker Deployment Guide - Payego

> Complete guide for deploying Payego using Docker and Docker Compose

## 📋 Overview

Payego's Docker setup includes:
- **Backend API** (Rust/Axum) with automatic migrations
- **PostgreSQL Database** with persistent storage
- **Prometheus** for metrics collection
- **Grafana** for metrics visualization

---

## 🚀 Quick Start

### Prerequisites
- Docker 20.10+
- Docker Compose 2.0+
- `.env` file configured (see below)

### 1. Environment Setup

Copy the example environment file and configure it:
```bash
cp .env.example .env
```

**Required environment variables:**
```env
# Database Configuration
DB_USER=postgres
DB_PASSWORD=your_secure_password_here
DB_NAME=payego
DB_PORT=5432

# Application Port
APP_PORT=8080

# JWT Configuration
JWT_SECRET=your_super_secret_key_must_be_at_least_32_characters_long
JWT_EXPIRATION_HOURS=2
ISSUER=payego-api
AUDIENCE=payego-client

# Payment Providers
STRIPE_SECRET_KEY=sk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...
PAYSTACK_SECRET_KEY=sk_test_...
PAYSTACK_WEBHOOK_SECRET=whsec_...
PAYPAL_CLIENT_ID=...
PAYPAL_SECRET=...

# Application Settings
RUST_LOG=info
CORS_ORIGINS=http://localhost:5173,http://localhost:3000
```

### 2. Build and Run

Start all services:
```bash
docker-compose up --build -d
```

The `-d` flag runs containers in detached mode (background).

### 3. Verify Deployment

Check that all services are running:
```bash
docker-compose ps
```

You should see 4 services running:
- `payego-app-1` (Backend API)
- `payego-db-1` (PostgreSQL)
- `payego-prometheus-1` (Metrics)
- `payego-grafana-1` (Dashboards)

---

## 🌐 Service Access

Once deployed, access the services at:

| Service | URL | Description |
|---------|-----|-------------|
| **Backend API** | http://localhost:8080 | Main application API |
| **Swagger UI** | http://localhost:8080/swagger-ui/ | API documentation |
| **Database** | localhost:5432 | PostgreSQL (use DB client) |
| **Prometheus** | http://localhost:9090 | Metrics collection |
| **Grafana** | http://localhost:3000 | Metrics dashboards (admin/admin) |

---

## 🏗️ Architecture

### Multi-Stage Build

The Dockerfile uses a **multi-stage build** for optimal image size:

1. **Builder Stage** (`rust:1.81-slim-bullseye`):
   - Installs build dependencies
   - Compiles Diesel CLI
   - Builds Rust application in release mode
   - Uses dependency caching for faster rebuilds

2. **Runtime Stage** (`debian:bullseye-slim`):
   - Minimal runtime dependencies
   - Non-privileged user (`appuser`)
   - Automatic database migrations on startup
   - ~200MB final image size

### Startup Sequence

When the container starts:
1. **Wait for Database**: Checks PostgreSQL readiness
2. **Run Migrations**: Applies all pending Diesel migrations
3. **Start Application**: Launches Payego API server

---

## 📦 Services Configuration

### Backend (app)

```yaml
ports:
  - "8080:8080"  # API server
environment:
  - DATABASE_URL=postgres://user:pass@db:5432/payego
  - JWT_SECRET=...
  - STRIPE_SECRET_KEY=...
  # ... other env vars
depends_on:
  - db
```

**Key Features:**
- Automatic database migrations
- Health checks via `pg_isready`
- Restart policy: `always`
- Custom DNS (8.8.8.8, 8.8.4.4)

### Database (db)

```yaml
image: postgres:15-alpine
ports:
  - "5432:5432"
volumes:
  - db_data:/var/lib/postgresql/data
```

**Key Features:**
- PostgreSQL 15 Alpine (lightweight)
- Persistent volume for data
- Automatic initialization

### Monitoring Stack

**Prometheus:**
- Collects metrics from Payego API
- Configuration: `prometheus.yml`
- Port: 9090

**Grafana:**
- Visualizes Prometheus metrics
- Default credentials: `admin/admin`
- Port: 3000

---

## 🔧 Common Operations

### View Logs

**All services:**
```bash
docker-compose logs -f
```

**Specific service:**
```bash
docker-compose logs -f app
docker-compose logs -f db
```

**Last 100 lines:**
```bash
docker-compose logs --tail=100 app
```

### Restart Services

**All services:**
```bash
docker-compose restart
```

**Specific service:**
```bash
docker-compose restart app
```

### Stop Services

```bash
docker-compose down
```

**Stop and remove volumes (⚠️ deletes database data):**
```bash
docker-compose down -v
```

### Database Access

**Connect to PostgreSQL:**
```bash
docker-compose exec db psql -U postgres -d payego
```

**Run SQL query:**
```bash
docker-compose exec db psql -U postgres -d payego -c "SELECT * FROM users LIMIT 5;"
```

### Run Migrations Manually

If you need to run migrations without restarting:
```bash
docker-compose exec app diesel migration run
```

### Execute Commands in Container

```bash
docker-compose exec app /bin/sh
```

---

## 🐛 Troubleshooting

### Container Won't Start

**Check logs:**
```bash
docker-compose logs app
```

**Common issues:**
- Missing environment variables → Check `.env` file
- Database not ready → Wait for `db` service to be healthy
- Port already in use → Change `APP_PORT` in `.env`

### Database Connection Errors

**Verify database is running:**
```bash
docker-compose ps db
```

**Check database logs:**
```bash
docker-compose logs db
```

**Test connection:**
```bash
docker-compose exec app pg_isready -h db -p 5432
```

### Migration Failures

**View migration status:**
```bash
docker-compose exec app diesel migration list
```

**Revert last migration:**
```bash
docker-compose exec app diesel migration revert
```

**Reset database (⚠️ destructive):**
```bash
docker-compose down -v
docker-compose up -d
```

### Application Crashes

**Check application logs:**
```bash
docker-compose logs --tail=200 app
```

**Common causes:**
- Invalid JWT_SECRET (must be 32+ characters)
- Missing payment provider credentials
- Database schema mismatch

### Port Conflicts

If port 8080 is already in use, change it in `.env`:
```env
APP_PORT=8081
```

Then restart:
```bash
docker-compose down
docker-compose up -d
```

---

## 🚢 Production Deployment

### Environment Variables

For production, ensure you:
1. Use strong, unique passwords
2. Set `RUST_LOG=warn` or `error` (not `debug`)
3. Configure proper `CORS_ORIGINS`
4. Use production payment provider keys
5. Set secure `JWT_SECRET` (32+ random characters)

### Security Considerations

**Database:**
- Change default PostgreSQL password
- Restrict database port exposure (remove from `ports` if not needed externally)
- Use encrypted connections

**Application:**
- Run behind reverse proxy (nginx/Traefik)
- Enable HTTPS/TLS
- Configure rate limiting
- Set up firewall rules

### Scaling

**Horizontal scaling:**
```bash
docker-compose up -d --scale app=3
```

**Note:** Requires load balancer configuration.

### Backup Database

**Create backup:**
```bash
docker-compose exec db pg_dump -U postgres payego > backup.sql
```

**Restore backup:**
```bash
cat backup.sql | docker-compose exec -T db psql -U postgres payego
```

---

## 🌍 Cloud Deployment

### Building for Different Platforms

If deploying to cloud with different CPU architecture (e.g., Mac M1 → AMD64 cloud):

```bash
docker build --platform=linux/amd64 -t payego:latest .
```

### Push to Registry

**Tag image:**
```bash
docker tag payego:latest your-registry.com/payego:latest
```

**Push to registry:**
```bash
docker push your-registry.com/payego:latest
```

### Example: AWS ECS

1. Push image to Amazon ECR
2. Create task definition with environment variables
3. Configure RDS PostgreSQL instance
4. Set up Application Load Balancer
5. Deploy ECS service

### Example: Google Cloud Run

1. Push image to Google Container Registry
2. Create Cloud SQL PostgreSQL instance
3. Deploy with environment variables
4. Configure Cloud Run service

---

## 📊 Monitoring

### Prometheus Metrics

Access Prometheus at http://localhost:9090

**Available metrics:**
- HTTP request duration
- Request count by endpoint
- Database connection pool stats
- System resource usage

### Grafana Dashboards

Access Grafana at http://localhost:3000

**Default credentials:** `admin/admin`

**Setup:**
1. Add Prometheus data source (http://prometheus:9090)
2. Import dashboards from `grafana/provisioning/`
3. Create custom dashboards as needed

---

## 🔄 Updates and Maintenance

### Update Application

1. Pull latest code
2. Rebuild and restart:
```bash
git pull
docker-compose up --build -d
```

### Update Dependencies

Rebuild with no cache:
```bash
docker-compose build --no-cache
docker-compose up -d
```

### Clean Up

**Remove unused images:**
```bash
docker image prune -a
```

**Remove unused volumes:**
```bash
docker volume prune
```

---

## 📚 Additional Resources

- [Docker's Rust Guide](https://docs.docker.com/language/rust/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [PostgreSQL Docker Image](https://hub.docker.com/_/postgres)
- [Diesel Migrations](https://diesel.rs/guides/getting-started.html)

---

## 💡 Tips

1. **Development:** Use `docker-compose up` (without `-d`) to see logs in real-time
2. **Production:** Always use `-d` flag and monitor logs separately
3. **Debugging:** Use `docker-compose exec app /bin/sh` to inspect container
4. **Performance:** Monitor Grafana dashboards for bottlenecks
5. **Backups:** Schedule regular database backups in production

---

## 🆘 Getting Help

If you encounter issues:
1. Check logs: `docker-compose logs app`
2. Verify environment variables in `.env`
3. Ensure all required services are running
4. Check [main README](README.md) for application-specific details
5. Review [Swagger UI](http://localhost:8080/swagger-ui/) for API documentation