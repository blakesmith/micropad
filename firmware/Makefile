all: micropad_debug.hex

target/thumbv6m-none-eabi/debug/micropad:
	cargo build

target/thumbv6m-none-eabi/release/micropad:
	cargo build --release

micropad_debug.hex: target/thumbv6m-none-eabi/debug/micropad
	cargo objcopy --target thumbv6m-none-eabi --bin micropad -- -O ihex $@

micropad_release.hex: target/thumbv6m-none-eabi/release/micropad
	cargo objcopy --release --target thumbv6m-none-eabi --bin micropad -- -O ihex $@


clean:
	rm -rf *.hex

.PHONY: clean
