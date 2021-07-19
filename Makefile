export COMMIT_HASH := $(shell git rev-parse --short=8 HEAD)
export PKG_CONFIG_ALLOW_CROSS=1

all: build docker-push

build:
	cargo build --release --target=x86_64-unknown-linux-musl

docker-image:
	docker build --no-cache -t andrewvwebber/cloudevents:${COMMIT_HASH} .

docker-push: docker-image
	docker push andrewvwebber/cloudevents:${COMMIT_HASH}
