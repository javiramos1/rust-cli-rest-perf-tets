TAG = 0.0.1

ARGS := $(filter-out run, $(MAKECMDGOALS))
bld:
	cargo build

build:
	DOCKER_BUILDKIT=1 docker build -t rust-cli/rust-cli-rest-perf-tests:$(TAG) .

run:
	docker run rust-cli/rust-cli-rest-perf-tests:$(TAG) ./rust-cli-rest-perf-tests $(ARGS)

push:
	docker push rust-cli/rust-cli-rest-perf-tests:$(TAG)
