.PHONY: \
	build run test lint fmt clean check-license test-coverage image-run \
	docs-build docs-serve \
	ci-check dev-setup setup

build:
	./scripts/cw-build.sh

run:
	./scripts/cw-run.sh $(ARGS)

test:
	./scripts/cw-test.sh

test-coverage:
	./scripts/cw-test-coverage.sh

lint:
	./scripts/cw-lint.sh

fmt:
	./scripts/cw-fmt.sh

clean:
	./scripts/cw-clean.sh

check-license:
	./scripts/cw-license.sh

image-run:
	./scripts/cw-image-run.sh $(ARGS)

docs-build:
	./scripts/docs-build.sh

docs-serve:
	./scripts/docs-serve.sh

ci-check:
	./scripts/ci-check.sh

setup:
	./scripts/setup.sh