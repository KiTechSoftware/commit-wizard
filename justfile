set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

build:
    ./scripts/cw-build.sh

run *args:
    ./scripts/cw-run.sh {{args}}

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

image-run *args:
    ./scripts/cw-image-run.sh {{args}}

docs-build:
    ./scripts/docs-build.sh

docs-serve:
    ./scripts/docs-serve.sh

ci-check:
    ./scripts/ci-check.sh

setup:
    ./scripts/setup.sh

crate-check:
    ./scripts/crate-release-check.sh