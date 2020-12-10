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

flash: micropad_release.hex
	./openocd_flash.sh $<


clean:
	rm -rf *.hex *.bin

.PHONY: clean setup flash
