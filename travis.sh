#!/bin/bash

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

run_tests_in() {
  cd $1
  cargo test
  cargo test --no-default-features
  cargo test --features backtrace
  cd $DIR
}

test_nightly_features_in() {
  cd $1
  cargo test --features small-error
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
