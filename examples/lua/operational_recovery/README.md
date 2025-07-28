# Operational Recovery Examples

This directory contains examples for operational recovery scenarios, including disaster recovery and backup validation procedures.

## Examples

### `disaster_recovery_procedure.lua`
Automated disaster recovery procedure that demonstrates:
- Step-by-step recovery process
- System health validation
- Configuration restoration
- Service recovery with rollback capabilities
- Recovery validation and monitoring

### `backup_validation.lua`
Comprehensive backup validation and monitoring that includes:
- Backup integrity checks
- Validation of backup chains
- Storage usage monitoring
- Backup age and retention validation
- Automated health reporting

## Configuration

Both examples require backup functionality:
- Use `examples/configs/backup-enabled.toml` for full backup API support
- Can use `examples/configs/state-enabled.toml` for manual recovery patterns

## Running the Examples

```bash
# Run disaster recovery procedure
./target/debug/llmspell -c examples/configs/backup-enabled.toml run examples/lua/operational_recovery/disaster_recovery_procedure.lua

# Run backup validation
./target/debug/llmspell -c examples/configs/backup-enabled.toml run examples/lua/operational_recovery/backup_validation.lua
```

## Key Features

### Disaster Recovery
- Simulates critical system failure scenarios
- Validates system components before recovery
- Provides rollback capability if recovery fails
- Includes comprehensive logging and reporting

### Backup Validation
- Checks backup integrity and completeness
- Validates backup chains for incremental backups
- Monitors backup storage usage
- Ensures compliance with retention policies

## Best Practices

1. **Regular Testing**: Run these scripts regularly to ensure recovery procedures work
2. **Documentation**: Keep recovery procedures documented and up-to-date
3. **Monitoring**: Set up automated backup validation on a schedule
4. **Practice**: Perform disaster recovery drills in test environments

## Notes

- These examples demonstrate recovery patterns without requiring actual system failures
- In production, integrate with your monitoring and alerting systems
- Customize validation criteria based on your specific requirements
- Consider automating these checks as part of your operational procedures