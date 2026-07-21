#!/usr/bin/env bash
set -euo pipefail

VERSION="2.6.0"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${ROOT_DIR}/oracle/v${VERSION}"
ZIP="${DEST}/WIDesigner-${VERSION}.zip"

mkdir -p "${DEST}"

if [[ ! -f "${ZIP}" ]]; then
  echo "Downloading WIDesigner ${VERSION} release zip..."
  curl -L -o "${ZIP}" \
    "https://github.com/edwardkort/WWIDesigner/releases/download/v${VERSION}/WIDesigner-${VERSION}.zip"
fi

echo "Unzipping into ${DEST}..."
unzip -q -o "${ZIP}" -d "${DEST}"

# The release zip nests everything under WIDesigner-2.6/; flatten so tests
# find oracle/v2.6.0/constraints/... etc. directly.
NESTED="${DEST}/WIDesigner-2.6"
if [[ -d "${NESTED}" ]]; then
  cp -R "${NESTED}/." "${DEST}/"
  rm -rf "${NESTED}"
fi

echo "Oracle ready at: ${DEST}"