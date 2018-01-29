#!/bin/bash

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

run_tests_in() {
  cd $1
  "echo run tests in $1"
  cargo test
  cargo test --no-default-features
  cd $DIR
}

main() {
  echo "begin main"
  run_tests_in "$DIR/failure-1.X"
  run_tests_in "$DIR/failure-0.1.X"
}

main
