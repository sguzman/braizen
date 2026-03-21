#!/usr/bin/env bash
set -euo pipefail

DEST="${1:-./vendor/servo}"
TAG="${BRAZEN_SERVO_TAG:-v0.0.4}"

if [[ -d "$DEST/.git" ]]; then
  echo "Servo already present at $DEST"
  exit 0
fi

echo "Cloning Servo tag ${TAG} into ${DEST}"
git clone --depth 1 --branch "${TAG}" https://github.com/servo/servo.git "${DEST}"
