#!/usr/bin/env bash

set -euo pipefail

detector_dir="$(mktemp -d)"

cleanup() {
	rm -rf "${detector_dir}"
}

trap cleanup EXIT

git clone \
	--quiet \
	--depth 1 \
	--branch v1 \
	https://github.com/gensecaihq/Shai-Hulud-2.0-Detector.git \
	"${detector_dir}"

docker run --rm \
	-v "$PWD:/workspace:ro" \
	-v "${detector_dir}:/detector:ro" \
	-w /workspace \
	node:24 \
	node /detector/dist/index.js \
		--fail-on-critical \
		--scan-lockfiles \
		--output-format text \
		--working-directory /workspace