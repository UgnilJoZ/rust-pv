#!/bin/sh
set -e

cargo build --release
echo
echo "Test 1: dd if=/dev/zero bs=10000 count=20000 2> /dev/null | target/release/pv -s 200000000 > testfile"
dd if=/dev/zero bs=10000 count=20000 2> /dev/null| target/release/pv -s 200000000 > testfile
echo
echo "Test 2: target/release/pv -f testfile > /dev/null"
target/release/pv -f testfile > /dev/null
echo
echo "Did this look good?"
unlink testfile
