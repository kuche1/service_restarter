#! /usr/bin/env bash

{
    set -euo pipefail

    ME=$(readlink -e -- "${BASH_SOURCE[0]}")
	HERE=$(dirname -- "$ME")

    rustc "$HERE"/service_restarter.rs
}