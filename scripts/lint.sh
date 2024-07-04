#!/bin/bash
set -eox pipefail

rustup component add clippy

run_clippy() {
    cargo clippy -p "$1" \
        -- \
        \
        -W clippy::all \
        -W clippy::pedantic \
        \
        -A clippy::module_name_repetitions \
        -A clippy::needless-pass-by-value \
        -A clippy::must-use-candidate \
        -A clippy::missing-panics-doc \
        -A clippy::wrong_self_convention \
        -A clippy::new_ret_no_self \
        \
        -D warnings
}

run_clippy multisig
run_clippy multisig-model
