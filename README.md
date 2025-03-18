# Palworld Server

[![Managed by GSM](https://img.shields.io/badge/Managed%20By-GSM-blue)](https://github.com/mbround18/game-server-management)

🌟 Welcome to the ultimate Palworld Server Setup! 🌍 This GitHub repository is your go-to toolkit 🛠️ for launching a Palworld server in a snap using Docker! Choose from preset worlds like 'casual' 🏖️, 'normal' 🌆, or 'hard' 🌋, or dive deep into customization with flexible settings 🎧.

## !!Notice!! Bug with saves, developers of Palworld working hard to fix!

With the [recent bug on save corruption](https://www.ign.com/articles/palworld-dev-working-to-fix-serious-bugs-including-lost-save-data),
we highly recommend you backup your save files! [Click here to see how to integrate auto backups.](./docs/compose-with-backups.md)

## Prerequisites

- Docker
- Docker Compose

## Configuration Options

The server can be configured either through environment variables or by passing arguments directly to the Docker container. The available presets are:
[They were configured based on this article](https://www.gtxgaming.co.uk/best-world-settings-for-palworld/)

- `casual`
- `normal`
- `hard`

### Environment Variables

Below is a list of available environment variables to customize your server:

#### General Server Settings

- `SERVER_NAME`: The name of your server.
- `SERVER_DESCRIPTION`: A short description of your server.
- `PUBLIC_IP`: Set the public IP of the server.
- `PUBLIC_PORT`: The public port for connections.
- `PORT`: The internal port of the game server.
- `ADMIN_PASSWORD`: The password for admin access.
- `SERVER_PASSWORD`: The password required to join the server.
- `REGION`: Define the server’s region.
- `USE_AUTH`: Enable authentication (`true` or `false`).
- `WEBHOOK_URL`: Discord webhook URL for server notifications.

#### Performance and Networking

- `MULTITHREADING`: Enable multithreading (`true` or `false`).
- `RCON_ENABLED`: Enable remote console (`true` or `false`).
- `RCON_PORT`: Port for RCON access.
- `RESTAPI_ENABLED`: Enable REST API (`true` or `false`).
- `RESTAPI_PORT`: Port for the REST API.
- `MAX_BUILDING_LIMIT_NUM`: Maximum number of buildings allowed.

#### Auto-Update Configuration

- `AUTO_UPDATE`: Enable automatic server updates (`true` or `false`).
- `AUTO_UPDATE_SCHEDULE`: Cron job format defining update checks (default: `0 3 * * *` for 3 AM daily updates).

#### Gameplay and Balance

- `EXP_RATE`: Modify experience rate multiplier.
- `PAL_CAPTURE_RATE`: Adjust the Pal capture success rate.
- `DAY_TIME_SPEED_RATE`: Adjust how fast daytime progresses.
- `NIGHT_TIME_SPEED_RATE`: Adjust how fast nighttime progresses.
- `DEATH_PENALTY`: Define what happens on player death.
- `ENABLE_FAST_TRAVEL`: Allow fast travel (`true` or `false`).
- `ENABLE_INVADER_ENEMY`: Enable invader enemies (`true` or `false`).
- `HARDCORE`: Enable hardcore mode (`true` or `false`).
- `PAL_LOST`: Determine if Pals are lost upon death (`true` or `false`).
- `ITEM_WEIGHT_RATE`: Adjust item weight multiplier.
- `PAL_DAMAGE_RATE_ATTACK`: Modify Pal attack damage.
- `PAL_DAMAGE_RATE_DEFENSE`: Modify Pal defense rate.
- `PLAYER_DAMAGE_RATE_ATTACK`: Modify player attack damage.
- `PLAYER_DAMAGE_RATE_DEFENSE`: Modify player defense rate.
- `WORK_SPEED_RATE`: Adjust work speed multiplier.
- `AUTO_SAVE_SPAN`: Set the frequency of autosaves (in minutes).

#### Multiplayer and Guild Settings

- `GUILD_PLAYER_MAX_NUM`: Maximum number of players per guild.
- `BASE_CAMP_MAX_NUM_IN_GUILD`: Maximum number of camps per guild.
- `ALLOW_CONNECT_PLATFORM`: Restrict platform connections (`Steam`, `Epic`, etc.).
- `SHOW_PLAYER_LIST`: Display the online player list (`true` or `false`).
- `CHAT_POST_LIMIT_PER_MINUTE`: Limit chat messages per minute.
- `EXIST_PLAYER_AFTER_LOGOUT`: Keep players visible after logout (`true` or `false`).
- `ENABLE_DEFENSE_OTHER_GUILD_PLAYER`: Enable defense against other guilds (`true` or `false`).

To see a full list of supported configuration options, see the [Environment Configuration Options](./docs/environment_variables.md) page.

## Using Docker Compose

To run the server with Docker Compose, create a `docker-compose.yml` file with the following content:

```yaml
version: "3.8"
services:
  palworld:
    image: mbround18/palworld-docker:latest
    environment:
      PRESET: "casual" # Options: casual, normal, hard
      MULTITHREADING: true # Enables multithreading
      PUBLIC_IP: "0.0.0.0"
      PUBLIC_PORT: "8211"
      SERVER_NAME: "My Palworld Server"
      EXP_RATE: "1.5"
      WEBHOOK_URL: "https://discord.com/api/webhooks/..."
      AUTO_UPDATE: "true"
      AUTO_UPDATE_SCHEDULE: "0 3 * * *"
    ports:
      - "8211:8211" # Default game port
      - "27015:27015" # Steam query port
    volumes:
      - "./data:/home/steam/palworld"
```

### Running the Server

To start the server with your chosen configuration, run:

```bash
docker-compose up
```

This command builds the Docker image if necessary and starts the server. The `PRESET` environment variable determines the server's configuration preset. You can also override any specific setting by adding it to the `environment` section of the `docker-compose.yml` file.

### Automatic Updates

If `AUTO_UPDATE` is enabled, the server will automatically check for updates at the scheduled time (`AUTO_UPDATE_SCHEDULE`). If an update is found, the server will:

1. Stop the running instance.
2. Download and apply the update.
3. Restart the server.

### Discord Webhook Notifications

If `WEBHOOK_URL` is set, the server will send notifications for:

- **Server Start & Stop** events.
- **Player Join & Leave** messages.
- **Server Updates**.

## Updating Server Settings

To update the server settings after initial setup, modify the `docker-compose.yml` file as needed and restart the server:

```bash
docker-compose down
docker-compose up
```

This ensures that your server configuration remains up to date.

## Contributions

Contributions to this project are welcome! Please submit a pull request or open an issue for any bugs, features, or improvements.
