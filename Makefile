.PHONY: docker-build docker-dev docker-push
GIT_TAG := $(shell git rev-parse --short HEAD)
export COMPOSE_BAKE=true

docker-build:
	@docker compose build

docker-dev: docker-build
	@docker compose up --abort-on-container-exit

docker-push: docker-build
	@docker tag mbround18/palworld-docker:latest mbround18/palworld-docker:sha-$(GIT_TAG)
	@docker image push mbround18/palworld-docker:sha-$(GIT_TAG)
	@echo "Pushed mbround18/palworld-docker:sha-$(GIT_TAG)"