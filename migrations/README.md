# Database Migrations

RedisGate uses SQLx migrations for database schema management, replacing the previous `init-db.sql` approach.

## Tables

The migration system creates the following tables:

1. **users** - User accounts for the RedisGate platform
2. **organizations** - Organizations/teams for resource management
3. **api_keys** - API keys for authentication and authorization
4. **redis_instances** - Redis instance configurations with comprehensive metadata including:
   - Network configuration (port, private_ip_address, public_ip_address, domain)
   - Resource management (max_memory, current_memory, cpu_usage_percent, memory_usage_percent)
   - Status tracking (status, health_status, last_health_check_at)
   - Kubernetes metadata (namespace, pod_name, service_name)
   - Backup and persistence settings
5. **organization_memberships** - Many-to-many relationship between users and organizations
6. **audit_logs** - Security and compliance audit trail

## Running Migrations

### Development

Migrations are automatically run when the application starts:

```bash
cargo run
```

### Manual Migration Commands

```bash
# Run pending migrations
sqlx migrate run

# Revert the last migration
sqlx migrate revert

# Check migration status
sqlx migrate info

# Create a new migration
sqlx migrate add <migration_name>
```

### Database Setup

1. Start PostgreSQL (via Docker Compose):
   ```bash
   docker compose up -d postgres
   ```

2. Set the DATABASE_URL environment variable:
   ```bash
   export DATABASE_URL="postgresql://redisgate_dev:redisgate_dev_password@localhost:5432/redisgate_dev"
   ```
   Or use the `.env` file (already configured).

3. Run migrations:
   ```bash
   sqlx migrate run
   ```

## SQLx Offline Mode

The project is configured for SQLx offline mode to support CI/CD environments without a database connection.

To prepare offline query data:
```bash
cargo sqlx prepare
```

This generates `.sqlx/query-*.json` files that should be committed to version control.

## Migration Files

All migration files are located in the `migrations/` directory and are executed in chronological order based on their timestamp prefixes.

## Development Workflow

1. Make schema changes by creating new migration files
2. Test migrations locally with `sqlx migrate run`
3. Generate offline query data with `cargo sqlx prepare` if needed
4. Commit both migration files and any generated `.sqlx/` files

## Replacing init-db.sql

The previous `scripts/init-db.sql` approach has been replaced by this migration system. The Docker Compose configuration has been updated to no longer run the init script automatically.