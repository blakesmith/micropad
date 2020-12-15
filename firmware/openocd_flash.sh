#!/bin/sh

OPENOCD=openocd
HEX=$1

$OPENOCD -f "/usr/share/openocd/scripts/interface/jlink.cfg" \
         -c "transport select swd" \
         -c "reset_config none separate" \
         -f "/usr/share/openocd/scripts/target/stm32f0x.cfg" \
         -c "init" \
         -c "reset halt" \
         -c "flash write_image erase $HEX" \
         -c "reset run" \
         -c "exit"


