# XP-Pen Deco L(W) "managed" mode HID interface

**WARNING: RISK OF BRICKING. DO NOT USE.**

I am using the term "managed mode" to describe the mode where the tablet uses vendor-specific HID packets to communicate with the "pentablet" driver.

All managed mode packets appear to use report id 0x02, consistent with the "vendor" usage (0x01) in the HID descriptor. Extra null bytes at end do not appear to matter.

`hidraw` looks to be able to communicate with the device only in USB mode. See section "bluetooth" for bluetooth mode. 3 `hidraw` devices are created, only the last one appears to work. TODO: figure out how this works.

## [`b0`] Switching to managed mode

```
                    bit 2, 3: managed ---+
        field type: 0xb0 -------------++ |
------------------------------        || |
                                      vv v
To switch into managed mode -- send 02b00400000000000000
To leave managed mode -------- send 02b00200000000000000
```

Other bits do not appear to be used?

## [`f2`] Battery status(?)

- `02`: report type
- `b0`: battery status
- `01`: unknown
- `64`: 100% battery
- `01`: charging

Tablet periodically sends what appears to be battery status.

```
                      bit 1: charging ---+
        field type: 0xf2 ---------++     |
-------------------------------   ||     |
                                  vv     v
Battery full, charging -------- 02f20164010000000000
Battery full, discharging ----- 02f20164000000000000
Discharging after a while ----- 02f20163000000000000
```

## [`f0`] Shortcut buttons

Appears to be an 8-bit bitfield, where first button (from top) is 0x01.

```
the bitfield -------++
field type: 0xf0 -++||
---------------   ||||
                  vvvv
Button 1 ------ 02f00100000000000000
Button 2 ------ 02f00200000000000000
Button 3 ------ 02f00400000000000000
Button 4 ------ 02f00800000000000000
Button 5 ------ 02f01000000000000000
Button 6 ------ 02f02000000000000000
Button 7 ------ 02f04000000000000000
Button 8 ------ 02f08000000000000000
Button 1, 2 --- 02f00300000000000000
```

## [`f8`] Wireless adapter stuff

Included wireless adapter will signal connected/disconnected status. On connect, must send managed mode again.

Wireless adapter seems to send 2 null bytes after every packet. Not sure why.

```
Disconnected ------- 02f804010000000000000000
Connected ---------- 02f802010000000000000000
```

## [`a0-a7`] Stylus


8192 levels of pressure corresponds to 13 bits.

Pen button status in lower 3 bits of type:

- `000`: no buttons
- `001`: pen tip pressed
- `010`: lower button pressed
- `100`: upper button pressed

Example: `02 a0 70c6 1077 fa2e 1329`

- `02`: report type
- `a0`: button state: hover only
- `70c6`: x-coord
- `1077`: y-coord
- `fa2e`: pen pressure(?)
- `1329`: tilt(?)

```
Top left corner, hover --------------------- 02a0000000000020faf3
Bottom right corner, hover ----------------- 02a070c6107700200f10
Top right corner, hover -------------------- 02a070c60000002013f8
Bottom left corner, hover ------------------ 02a0000010770020ea27
Top left, hover, tilt mostly left ---------- 02a0000000000020c4df
Top left, hover, tilt mostly up ------------ 02a0000000000020ddfb
Top left, hover, tilt mostly right --------- 02a00000000000203c31
Top left, hover, tilt mostly down ---------- 02a00000000000200717
Top left, hover, less tilt ----------------- 02a00000000000201329
Bottom left, pen tip ----------------------- 02a100001077fa2edf22
Bottom left, lower button ------------------ 02a2000010770020d91b
Bottom left, pen tip and lower button ------ 02a300001077e72ece27
Bottom left, upper button ------------------ 02a4000010770020d3fe
```

## [`c0`] Pen left(?)

```
Pen leaves surface ----------- 02c00000eb00002000000000
```

## Bluetooth stuff

Non-managed mode HID reports are sent over handle 0x0020 (UUID type: Report). Host-to-device managed mode packets are sent over handle 0x002e (UUID type: SDP). Device-to-host managed mode packets are sent over handle 0x0033 (UUID type: RFCOMM). Despite having type RFCOMM, it does not use RFCOMM.

`org.bluez /org/bluez/hci0/dev_[addr]/service002c/char002d org.bluez.GattCharacteristic1 WriteValue([0x02, 0xb0, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], {})` (dbus) is sufficient to put the device in managed mode (protocol same as above).

Acquire notify on `/org/bluez/hci0/dev_[addr]/service002c/char0032` to get the protocol identical to what is described above.


