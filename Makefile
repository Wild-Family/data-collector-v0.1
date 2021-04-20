# these will speed up builds, for docker-compose >= 1.25
export COMPOSE_DOCKER_CLI_BUILD=1
export DOCKER_BUILDKIT=1

all: down build up test

build:
	docker-compose build

up:
	docker-compose up -d

down:
	docker-compose down --remove-orphans

restart: down up

test: up
	docker-compose run --rm --no-deps --entrypoint=pytest 

unit-tests:
	docker-compose run --rm --no-deps --entrypoint=pytest app /tests/unit

integration-tests: up
	docker-compose run --rm --no-deps --entrypoint=pytest app /tests/integration

e2e-tests: up
	docker-compose run --rm --no-deps --entrypoint=pytest app /tests/e2e

bash:
	docker-compose exec app bash

logs:
	docker-compose logs app | tail -100

db-logs:
	docker-compose logs db | tail -100

pgadmin-logs:
	docker-compose logs pgadmin4 | tail -100


init:
	mkdir grafana pgadmin postgres/data