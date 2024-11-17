#!/usr/bin/env bash

CRATES=(
  internal
  derive
  public
)

DRY_RUN=${DRY_RUN:-1}

function die {
  echo "$*" >/dev/stderr
  exit 42
}

if [ ! "$DRY_RUN" = "1" ]; then
  while true; do
    echo -n -e "\x1b[1;33mnotice\x1b[0m you are about to publish ${#CRATES[@]} crates, continue ? [yN] "
    read -r ANS
    case $ANS in
      y|Y) break;;
      n|N) die "aborted" ;;
      *) echo -e "\x1b[0;31mHuh ?\x1b[0m";;
    esac
  done
fi

for CRATE in ${CRATES[@]}; do
  echo -e "---------------- Crate \x1b[1m$CRATE\x1b[0m ----------------------"
  ARGS=
  if [ "$DRY_RUN" = "1" ]; then
    ARGS="$ARGS --dry-run"
  fi
  $(cd $CRATE && cargo publish $ARGS) || die "failed to publish $CRATE"
done

exit 0