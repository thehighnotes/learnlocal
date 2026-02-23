# Service Recovery

When a service won't start, the answer is almost always in one of four places:
configuration, dependencies, environment, or the deploy itself. Knowing where to
look -- and in what order -- turns a 2-hour mystery into a 10-minute fix.


## Configuration Errors

The most common reason a service fails to start: a bad config file. JSON missing
a comma. YAML with a tab where spaces were expected. An INI file with a typo in a
section header.

The error log usually tells you _exactly_ what's wrong:

```
[ERROR] Failed to parse /etc/app/config.json: Unexpected token } at line 14
```

Always validate config before restarting a service:

```
python3 -c "import json; json.load(open('config.json'))"   # JSON
ruby -e "require 'yaml'; YAML.load_file('config.yaml')"     # YAML
nginx -t                                                      # nginx config
```

If the validator says it's fine, the problem isn't syntax -- it's semantics.
A valid JSON file with the wrong database hostname is still broken.


## Dependency Checks

Services depend on things: files, directories, other services, network endpoints.
When a dependency is missing, the error is usually clear:

```
FileNotFoundError: /var/lib/app/cache.db
Connection refused: localhost:5432
```

The fix varies:
- **Missing file/directory**: create it (`touch`, `mkdir -p`)
- **Missing service**: start the dependency first
- **Missing package**: install it

Check dependencies _before_ trying to start the service. Read the startup script
or systemd unit file to understand what it expects.


## Environment Variables

Modern applications pull configuration from the environment. A missing env var
is invisible until the app tries to read it:

```
KeyError: 'DATABASE_URL'
Error: Required environment variable API_KEY is not set
```

Common patterns for setting env vars:
- `.env` files loaded by the app or a process manager
- `/etc/environment` for system-wide vars
- `export VAR=value` in shell profiles
- systemd `EnvironmentFile=` directives

A `.env` file typically looks like:

```
DATABASE_URL=postgres://localhost:5432/myapp
REDIS_URL=redis://localhost:6379
SECRET_KEY=change-me-in-production
```


## Rollback Strategies

When a new deploy breaks things, the fastest fix is often to go back:

- **Symlink swaps**: `current` points to a release directory. Roll back by
  relinking to the previous version.
- **Package managers**: `apt install app=1.9.0`, `yum downgrade app`
- **Container tags**: `docker run app:v1.9` instead of `app:v2.0`
- **Git**: `git checkout v1.9.0` in the deploy directory

The symlink pattern is the most common for custom applications:

```
/opt/app/
  releases/
    v1.8/
    v1.9/    <-- known good
    v2.0/    <-- broken
  current -> releases/v2.0    # change this to v1.9
```

```
ln -sfn releases/v1.9 current    # atomic symlink swap
```


## Health Checks

After fixing something, don't just assume it works. Verify:

```bash
curl -f http://localhost:8080/health    # HTTP health endpoint
systemctl is-active myapp              # systemd status
pg_isready -h localhost                # PostgreSQL
redis-cli ping                         # Redis
```

A good health check script verifies all critical dependencies and gives a
clear OK/FAIL result. Automate what you'd check manually.


## The Recovery Checklist

When a service is down, work through this in order:

1. **Read the error log** -- it usually tells you what's wrong
2. **Check the config** -- syntax and content
3. **Verify dependencies** -- files, dirs, services, network
4. **Check environment** -- env vars, permissions, disk space
5. **Try a restart** -- sometimes that's all it takes
6. **Roll back** -- if the new version is the problem
7. **Health check** -- verify the fix actually worked

Don't skip steps. Don't guess. Be methodical.
