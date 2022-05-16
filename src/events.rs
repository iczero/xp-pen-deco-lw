use bytes::{Buf, Bytes};
use anyhow::anyhow;

#[derive(Debug)]
pub enum Event {
    /// Battery status updates
    BatteryStatus {
        /// Battery level, from 0 to 100
        level: u8,
        charging: bool
    },

    /// Shortcut button status
    ShortcutButtons {
        /// Bitfield (8 bits) of pressed buttons
        ///
        /// WARNING: DO NOT PRESS ALL 8 BUTTONS SIMULTANEOUSLY OR YOU MAY BRICK THE THING
        buttons: u8,
    },

    /// Stylus movement updates
    Stylus {
        tip_pressed: bool,
        lower_button: bool,
        upper_button: bool,
        x_pos: u16,
        y_pos: u16,
        tip_pressure: u16,
        tilt_x: i8,
        tilt_y: i8
    },

    /// Stylus gone
    PenLeft {
        x_pos: u16,
        y_pos: u16
    },

    /// Command specific to included USB "bluetooth" adapter.
    WirelessAdapterStatus {
        connected: bool
    }
}

impl Event {
    pub fn parse(buf_slice: &[u8]) -> anyhow::Result<Self> {
        if buf_slice.len() < 10 {
            return Err(anyhow!("Buffer too short"));
        }
        let mut buf = Bytes::copy_from_slice(buf_slice);
        let report_type = buf.get_u8();
        if report_type != 2 {
            return Err(anyhow!("Invalid report type"));
        }
        let field_type = buf.get_u8();
        match field_type {
            0xf2 => {
                // battery status
                let _ = buf.get_u8();
                let level = buf.get_u8();
                let charging = buf.get_u8() == 1;
                Ok(Event::BatteryStatus {
                    level,
                    charging
                })
            },
            0xf0 => {
                // shortcut buttons
                let buttons = buf.get_u8();
                Ok(Event::ShortcutButtons { buttons })
            },
            0xa0..=0xa7 => {
                let tip_pressed = (field_type & 0b001) != 0;
                let lower_button = (field_type & 0b010) != 0;
                let upper_button = (field_type & 0b100) != 0;
                let x_pos = buf.get_u16_le();
                let y_pos = buf.get_u16_le();
                // dunno why the subtract 8192 is needed
                let tip_pressure = buf.get_u16_le() - 8192;
                let tilt_x = buf.get_i8();
                let tilt_y = buf.get_i8();
                Ok(Event::Stylus {
                    tip_pressed,
                    lower_button,
                    upper_button,
                    x_pos,
                    y_pos,
                    tip_pressure,
                    tilt_x,
                    tilt_y
                })
            },
            0xc0 => {
                // pen left
                let x_pos = buf.get_u16_le();
                let y_pos = buf.get_u16_le();
                Ok(Event::PenLeft { x_pos, y_pos })
            },
            0xf8 => {
                // wireless adapter status
                // no idea what most of this means
                let byte1 = buf.get_u8();
                let connected = byte1 & 0b10 != 0;
                Ok(Event::WirelessAdapterStatus { connected })
            }
            _ => Err(anyhow!("Unhandled type"))
        }
    }
}
