#!/bin/sh

cargo build --release
rm out.stacks

circom=target/release/circom
command="$circom $*"
sudo dtrace -c "$command" -o out.stacks -n 'profile-997 /execname == "circom"/ { @[ustack(100)] = count(); }'
./stackcollapse.pl out.stacks | ./flamegraph.pl > flamegraph.svg
open flamegraph.svg