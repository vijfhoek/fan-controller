#!/bin/bash
set -ex

cargo build --release
openocd -f openocd.cfg -c 'program target/thumbv6m-none-eabi/release/boffan verify reset exit'
