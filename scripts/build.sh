#!/usr/bin/env bash

wget -cq https://cdn.kernel.org/pub/linux/kernel/v${1%.*.*}.x/linux-$1.tar.xz
tar -xf linux-$1.tar.xz
pushd linux-$1
make defconfig
make modules_prepare
popd
export KERNEL_VERSION=$1
export KERNEL_SOURCE=$(pwd)/linux-$1
cargo build -p firewall --bin firewall && cargo build -p firewall --bin fw
rm -R linux-$1{,.tar.xz}
mkdir -p bin
mv target/debug/firewall ./bin/firewall-$1
mv target/debug/fw ./bin/fw
cargo clean
