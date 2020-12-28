# Micropad Serial Protocol

The micropad serial protocol is a simple byte-oriented format to
cotrol and configure the micropad from the connected host computer. It
uses a standard USB-CDC communication driver.

The first byte indicates the message type, followed by message
arguments. Successful responses return a 0 byte, error codes
non-zero. All message frames are 8 byte aligned (64 bit frames).

## Message Types

### 0x01 - Ping

*Description*: Ping the micropad.
*Arguments*: No arguments.
*Valid responses*

- 0: Success

### 0x02 - Set LED pulse brightness

*Description*: Set the LED pulse brightness when keys are pressed
*Arguments*: 1 byte brightness level.

- Arg 1: Brightness level. 0x00 - 0xFF.

*Valid responses*

- 0: Success

### 0x03 - Change RGB LED Color

*Description*: Set the RGB LED to the specified static color triplet.
*Arguments*: 3 byte hex color triplet.

- Arg 1: Red value.   0x00 - 0xFF.
- Arg 2: Green value. 0x00 - 0xFF.
- Arg 3: Blue value.  0x00 - 0xFF.

*Valid responses*

- 0: Success
- 1: The LED is disabled. No change.
