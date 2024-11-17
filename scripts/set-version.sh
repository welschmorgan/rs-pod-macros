#!/usr/bin/env bash

CARGO_ROOT_MANIFEST=Cargo.toml
CARGO_MANIFESTS=$(find . -mindepth 1 -not -path '**/target/**' -name 'Cargo.toml' | sort)

CUR_VER="$(cd internal && cargo read-manifest | jq -r '.version')"
NEW_VER=$1

function die {
  echo -e "$*" >/dev/stderr
  exit 42
}

[ -n "$NEW_VER" ] || die "usage: set-version.sh <NEW_VERSION>"
[ -n "$CUR_VER" ] || die "Failed to retrieve internal package version"

[ ! "$CUR_VER" = "$NEW_VER" ] || die "NEW_VER (=$NEW_VER) is same as CUR_VER (=$CUR_VER)"

perl -pe "s/version = \"${CUR_VER}\"/version = \"${NEW_VER}\"/" -i "$CARGO_ROOT_MANIFEST"
perl -pe "s/$CUR_VER/$NEW_VER/" -i "README.md"

for MANIFEST in ${CARGO_MANIFESTS[@]}; do
  perl -pe "s/podstru-(\w+) = (.*)version = \"${CUR_VER}\"/podstru-\1 = \2version = \"${NEW_VER}\"/g" -i "$MANIFEST"
done

exit 0