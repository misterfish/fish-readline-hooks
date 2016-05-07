#!/bin/bash -e

bindir=$(realpath "$(dirname "$0")")

srcdir=$(realpath "$bindir/src")
libdir=$(realpath "$bindir/lib")
targetdir=$(realpath "$bindir/target")

. "$bindir"/functions

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

chd "$srcdir/rh-parse"
# --- -d to also make header.
cmd bison --report=state -d rh-parse.y
cmd $GCC -c rh-parse.c
cmd $GCC -c rh-parse.tab.c

chd "$libdir"
chd fish-lib-util
cmd make

chd "$libdir"
cmd ar r librh-parse.a \
    "$srcdir/rh-parse"/rh-parse.o \
    "$srcdir/rh-parse"/rh-parse.tab.o \
    fish-lib-util/fish-util/fish-util.o

cmd mkdir -p "$targetdir"
chd "$targetdir"
for build in release debug; do
    cmd mkdir -p "$build/deps"
    chd "$build/deps"
    cmd ln -sf ../../../lib/librh-parse.a
    chd ../..
done
