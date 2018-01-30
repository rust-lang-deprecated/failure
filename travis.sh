#!/bin/bash

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cargo_test() {
   cargo test "$@" || { exit 101; }
}

run_tests_in() {
  cd $1
  cargo_test
  cargo_test --no-default-features
  cargo_test --features backtrace
  cd $DIR
}

test_nightly_features_in() {
  cd $1
  cargo_test --features small-error
  cargo_test --all-features
  cd $DIR
}

main() {
  run_tests_in "$DIR/failure-1.X"
  run_tests_in "$DIR/failure-0.1.X"
  if [ "${TRAVIS_RUST_VERSION}" = "nightly" ]; then
    test_nightly_features_in "$DIR/failure-1.X"
  fi
}

main
