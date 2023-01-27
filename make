#!/usr/bin/env bash

set -eu
set -o pipefail

rootdir=$(realpath "$(dirname "$0")")
srcdir=$(realpath "$rootdir/src")
libdir=$(realpath "$rootdir/lib")
targetdir=$(realpath "$rootdir/target")
fishutildir="$libdir"/fish-lib-util

. "$rootdir"/functions

USAGE="Usage: $0"

while getopts hb:-: arg; do
    case $arg in
        h) warn "$USAGE"; exit 0 ;;
        #b) ARG_B="$OPTARG" ;;
        -) LONG_OPTARG="${OPTARG#*=}"
            case $OPTARG in
                help)  warn "$USAGE"; exit 0 ;;
                #letter-b=?*)    ARG_B="$LONG_OPTARG" ;;
                #letter-b*)        error "Option '--$OPTARG' requires an argument" ;;
                #letter-a* | letter-c* ) error "Option '--$OPTARG' doesn't allow an argument" ;;
                '')    break ;; # "--" terminates argument processing
                *)     error "Illegal option --$OPTARG" ;;
                esac
        ;;
    esac
done
shift $((OPTIND-1))

GCC="gcc -fPIC -std=c99"

do-gcc () {
  cmd gcc -fPIC -std=c99 "$@"
}

make-parser () {
  # --- -d to also make header.
  cmd bison --report=state -d rh-parse.y
  fun do-gcc -I"$fishutildir" -c rh-parse.c
  fun do-gcc -I"$fishutildir" -c rh-parse.tab.c
  cmd ar r librh-parse.a \
    "$srcdir/rh-parse"/rh-parse.o \
    "$srcdir/rh-parse"/rh-parse.tab.o \
    "$fishutildir"/fish-util/fish-util.o
}

make-build-dirs () {
  local build
  for build in release debug; do
    cmd mkdir -p "$build/deps"
    cwd "$build/deps" ln -sf "$rootdir"/lib/librh-parse.a
  done
}

cwd "$libdir"/fish-lib-util make
cwd "$srcdir/rh-parse" make-parser
cmd mkdir -p "$targetdir"
cwd "$targetdir" make-build-dirs
