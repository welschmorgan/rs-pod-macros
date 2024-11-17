#!/usr/bin/env bash

FOLDERS=(
  $(find . -maxdepth 1 -type d -not -iname '\.git' -a -not -name '\.' -a -not -name 'target' -a -not -name 'scripts' -printf '%f\n')
)

WORKDIR=$(mktemp -d -t PROD-MACROS-XXXXXX)
LIST_FILE="$WORKDIR/examples.list"
BUILD_FILE="$WORKDIR/examples-build.log"

mkdir -p "$WORKDIR"
readarray -t FOLDERS<<<${FOLDERS[@]}

function die {
  echo -e "\x1b[1;31mfatal\x1b[0m: $*"
  exit 42
}

function build {
  RUSTFLAGS=-Awarnings cargo b # &>"$BUILD_FILE"
  STATUS=$?
  return $STATUS
  # if [ ! $STATUS = 0 ]; then
  #   echo "BAD STATUS: $STATUS"
  #   less "$BUILD_FILE" -f
  # fi
}

function build_example {
  RUSTFLAGS=-Awarnings cargo r --example "$1" # &>"$BUILD_FILE"
  STATUS=$?
  return $STATUS
}

function list_examples {
  bash -c "cd \"$1\" && cargo read-manifest | jq '.targets | map(select(.kind == [\"example\"]) | .name)' | grep '\"' | perl -pe 's/[ ,\"]+//g'" > "$LIST_FILE"
}

for FOLDER in ${FOLDERS[@]}; do
  build || die "failed to build workspace"
  list_examples "$FOLDER"
  for EXAMPLE in $(cat "$LIST_FILE"); do
    echo -e "---------- run: \x1b[90mexample\x1b[0m: \x1b[1m$EXAMPLE\x1b[0m (\x1b[4mworkdir\x1b[0m: $(pwd)) ----------"
    build_example "$EXAMPLE" || die "failed to build example '$EXAMPLE'"
  done
done

rm -f "$BUILD_FILE"