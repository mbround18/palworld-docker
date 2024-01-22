# Palworld Server 

This repository contains a Dockerized application for configuring and running a Palworld server with customizable settings. You can easily choose between different server presets (`casual`, `normal`, `hard`) or manually configure individual settings to tailor the server to your preferences.

> NOTE: WHEN starting your server with a custom configuration, it will be permanent and you cannot change it. this is a bug currently and we are waiting on the devs to fix it. 

## Prerequisites

- Docker
- Docker Compose

## Configuration Options

The server can be configured either through environment variables or by passing arguments directly to the Docker container. The available presets are:
[They were configured based on this article](https://www.gtxgaming.co.uk/best-world-settings-for-palworld/)

- `casual`
- `normal`
- `hard`

Additionally, you can customize the following settings:

- `DAY_TIME_SPEED_RATE`: Control the speed of day time.
- `NIGHT_TIME_SPEED_RATE`: Control the speed of night time.
- `EXP_RATE`: Experience points rate.
- (And so on for each configurable option...)

To see a full list of supported configuration options, see the [Environment Configuration Options](./docs/environment_variables.md) page.

## Using Docker Compose

To run the server with Docker Compose, you first need to create a `docker-compose.yml` file in the root of this repository with the following content:

```yaml
version: "3.8"
services:
  palworld-server:
    image: mbround18/palworld-docker:latest
    environment:
      PRESET: "casual" # Options: casual, normal, hard
      # Optionally override specific settings:
      # DAY_TIME_SPEED_RATE: '1'
      # NIGHT_TIME_SPEED_RATE: '1'
      # And so on...
    ports:
      - "8211:8211" # Default game port
```

### Running the Server

To start the server with your chosen configuration, run:

```bash
docker-compose up
```

This command builds the Docker image if necessary and starts the server. The `PRESET` environment variable determines the server's configuration preset. You can also override any specific setting by adding it to the `environment` section of the `docker-compose.yml` file.

### Custom Configuration

If you wish to customize the server beyond the provided presets, simply add or modify the environment variables in the `docker-compose.yml` file. For example, to set a custom experience rate, you would add:

```yaml
environment:
  EXP_RATE: "1.5"
```

## Updating Server Settings

To update the server settings after initial setup, modify the `docker-compose.yml` file as needed and restart the server:

```bash
docker-compose down
docker-compose up
```

This process ensures that your server configuration is always up to date with your specifications.

## Contributions

Contributions to this project are welcome! Please submit a pull request or open an issue for any bugs, features, or improvements.
