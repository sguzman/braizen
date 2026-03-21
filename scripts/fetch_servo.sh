#!/usr/bin/env bash
set -euo pipefail

DEST="${1:-./vendor/servo}"
TAG="${BRAZEN_SERVO_TAG:-v0.0.4}"
REV="${BRAZEN_SERVO_REV:-}"

if [[ -d "$DEST/.git" ]]; then
  echo "Servo already present at $DEST"
  exit 0
fi

echo "Cloning Servo into ${DEST}"
git clone --depth 1 --branch "${TAG}" https://github.com/servo/servo.git "${DEST}"

if [[ -n "$REV" ]]; then
  echo "Checking out Servo revision ${REV}"
  pushd "${DEST}" >/dev/null
  git fetch --depth 1 origin "${REV}"
  git checkout "${REV}"
  popd >/dev/null
fi
