# SQLX Commands

This file describes the commands used to perform migration with sqlx.

---
### Make sure SQLx CLI is installed
```aiignore
cargo install sqlx-cli --no-default-features --features postgres
```

### 1: Create Migrations
```aiignore
sqlx migrate add -r <migration_name>
```

### 2: Export DATABASE_URL
```bash
export DATABASE_URL=postgres://user:labPW@localhost:5432/deviceDB
```

### 3: Apply migration
```aiignore
sqlx migrate run
```

### 4: Revert the last migration
```aiignore
sqlx migrate revert
```

### 5: Migration info
```aiignore
sqlx migrate info
```