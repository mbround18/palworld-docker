# Compose with Backups

> Seeing this error `ValueError: Input folder does not exist or is not a directory.` is normal if you are starting a new server for the first time.
> It just means no saves have been recorded yet.

[Click here to see all options for the backup cron](https://github.com/mbround18/backup-docker)

```yaml
version: "3.8"
services:
  palworld:
    image: mbround18/palworld-docker:latest
    environment:
      PRESET: "casual" # Options: casual, normal, hard
      # Optionally override specific settings:
      # DAY_TIME_SPEED_RATE: '1'
      # NIGHT_TIME_SPEED_RATE: '1'
      # And so on...
    ports:
      - "8211:8211" # Default game port
      - "27015:27015" # steam query port
    volumes:
      - ./data:/home/steam/palworld
  backups:
    image: mbround18/backup-cron:latest
    environment:
      - SCHEDULE=*/5 * * * *
      - INPUT_FOLDER=/home/steam/palworld/Pal/Saved/
      - OUTPUT_FOLDER=/home/steam/backups
      - OUTPUT_USER=1000
      - OUTPUT_GROUP=1000
      - RETAIN_N_DAYS=5
    volumes:
      - ./data:/home/steam/palworld
      - ./backups:/home/steam/backups
    restart: unless-stopped
```
