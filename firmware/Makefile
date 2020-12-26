all: micropad_release.hex

setup:
	cargo install cargo-binutils
	rustup target add thumbv6m-none-eabi
	rustup component add llvm-tools-preview

target/thumbv6m-none-eabi/debug/micropad:
	cargo build

target/thumbv6m-none-eabi/release/micropad:
	cargo build --release

micropad_release.bin: target/thumbv6m-none-eabi/release/micropad
	cargo objcopy --release --target thumbv6m-none-eabi --bin micropad -- -O binary $@

micropad_debug.hex: target/thumbv6m-none-eabi/debug/micropad
	cargo objcopy --target thumbv6m-none-eabi --bin micropad -- -O ihex $@

micropad_release.hex: target/thumbv6m-none-eabi/release/micropad
	cargo objcopy --release --target thumbv6m-none-eabi --bin micropad -- -O ihex $@

micropad_release.dfu: micropad_release.bin
	cp $< $@
	dfu-suffix -v 0483 -p df11 -d 22.00 -a $@ # STM32 bootloader ids
	dfu-prefix -L -s 0x08000000 -a $@

flash: micropad_release.hex
	./openocd_flash.sh $<

dfu_flash: micropad_release.dfu
	dfu-util -a 0 -s 0x08000000:leave -D $<

clean:
	rm -rf *.hex *.bin *.dfu

.PHONY: clean setup flash dfu_flash
