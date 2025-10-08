#!/usr/bin/env bash

# Program: update_llzk_lib.sh
# Description: This script updates the llzk-lib dependency
#   in the cargo lock file and the nix flake lock file.
#
# Required Programs:
#   - cargo: For updating the rust dependencies
#   - nix: For updating the nix flake
#
# Usage: ./scripts/update_llzk_lib.sh

set -e

cargo update -p llzk
nix flake update llzk-lib
nix flake update llzk-rs-pkgs
