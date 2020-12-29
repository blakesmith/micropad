# Micropad Serial Protocol

The micropad serial protocol is a simple byte-oriented format to
cotrol and configure the micropad from the connected host computer. It
uses a standard USB-CDC communication driver.

The first byte indicates the message type, followed by message
arguments. Successful responses return a 0 byte, error codes
non-zero. All message frames (requests and responses) are 8 byte aligned (64 bit frames).

Responses contain a response code in the first byte of the message
frame. Responses can also contain response arguments in the message
frame bytes, following the response code. The basic response codes
are:

0x00: Success.

0x80: Success, with a continuation bit set. The micropad will send
this response code when it has more response frames to send
callers. Callers continue to read message frames until either a
success response without the continuation bit set (0x00), or another
error value.

0x04: Not found.


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

### 0x03 - Get the LED pulse brightness

*Description*: Retrieve the LED pulse brightness when the keys are pressed.
*Arguments*: No arguments.

*Valid responses*

- 0: Success, with follow on response bytes.
  - Byte 2: The current brightness level. 0x00 - 0xFF.

### 0x04 - Get current mode information

*Description*: Retrieve information about the mode.
*Arguments*: No arguments.
*Valid Responses*:

- 0: Success, with follow on response bytes.
  - Byte 2: Built in mode count.
  - Byte 3: User configurable mode count.
  - Byte 4: Whether the current mode is a built-in mode, or user configurable mode. 0x00 - built-in (not writeable), 0x01 - user-configurable.
  - Byte 5: Current mode index. Enumeration starts at index 0, indexing the built-in modes first, followed by all the user modes. For example, if the built-in mode count is 2, and the user configurable mode count is 1, indices 0-1 would be built-in modes, and index 2 would be the user configurable mode.

