/*
 * Play beep sounds via the classic PC speaker.
 * The initial implementation programs the PIT directly.
 * Once we have an interrupt-based driver for the PIT,
 * we can simplify the PC speaker code to rely on the global system time instead.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf, 2016-09-22
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-03-18
 * License: GPLv3
 */

use crate::device::cpu::IoPort;
use crate::device::pit;
use crate::library::spinlock::Spinlock;

pub static SPEAKER: Spinlock<Speaker> = Spinlock::new(Speaker::new());

/// Driver struct for the PC speaker.
pub struct Speaker {
    pit_ctrl_port: IoPort,
    pit_data0_port: IoPort,
    pit_data2_port: IoPort,
    ppi_port: IoPort,
}

// Frequency of musical notes
// (Our OS does not really support floating point, so we convert the numbers to usize)
pub const C0: usize = 130.81 as usize;
pub const C0X: usize = 138.59 as usize;
pub const D0: usize = 146.83 as usize;
pub const D0X: usize = 155.56 as usize;
pub const E0: usize = 164.81 as usize;
pub const F0: usize = 174.61 as usize;
pub const F0X: usize = 185.00 as usize;
pub const G0: usize = 196.00 as usize;
pub const G0X: usize = 207.65 as usize;
pub const A0: usize = 220.00 as usize;
pub const A0X: usize = 233.08 as usize;
pub const B0: usize = 246.94 as usize;

pub const C1: usize = 261.63 as usize;
pub const C1X: usize = 277.18 as usize;
pub const D1: usize = 293.66 as usize;
pub const D1X: usize = 311.13 as usize;
pub const E1: usize = 329.63 as usize;
pub const F1: usize = 349.23 as usize;
pub const F1X: usize = 369.99 as usize;
pub const G1: usize = 391.00 as usize;
pub const G1X: usize = 415.30 as usize;
pub const A1: usize = 440.00 as usize;
pub const A1X: usize = 466.16 as usize;
pub const B1: usize = 493.88 as usize;

pub const C2: usize = 523.25 as usize;
pub const C2X: usize = 554.37 as usize;
pub const D2: usize = 587.33 as usize;
pub const D2X: usize = 622.25 as usize;
pub const E2: usize = 659.26 as usize;
pub const F2: usize = 698.46 as usize;
pub const F2X: usize = 739.99 as usize;
pub const G2: usize = 783.99 as usize;
pub const G2X: usize = 830.61 as usize;
pub const A2: usize = 880.00 as usize;
pub const A2X: usize = 923.33 as usize;
pub const B2: usize = 987.77 as usize;
pub const C3: usize = 1046.50 as usize;

#[repr(u16)]
/// I/O port addresses for the PC speaker.
enum SpeakerRegister {
    Data0 = 0x40,
    Data2 = 0x42,
    Control = 0x43,
    PPI = 0x61,
}

/// Base frequency of the PIT (Programmable Interval Timer) in Hz.
const PIT_FREQUENCY: usize = 1193180;

// Bits         Usage
// 7 and 6      Select channel :
//                 0 0 = Channel 0
//                 0 1 = Channel 1
//                 1 0 = Channel 2
//                 1 1 = Read-back command (8254 only)
// 5 and 4      Access mode :
//                 0 0 = Latch count value command
//                 0 1 = Access mode: lobyte only
//                 1 0 = Access mode: hibyte only
//                 1 1 = Access mode: lobyte/hibyte
// 3 to 1       Operating mode :
//                 0 0 0 = Mode 0 (interrupt on terminal count)
//                 0 0 1 = Mode 1 (hardware re-triggerable one-shot)
//                 0 1 0 = Mode 2 (rate generator)
//                 0 1 1 = Mode 3 (square wave generator)
//                 1 0 0 = Mode 4 (software triggered strobe)
//                 1 0 1 = Mode 5 (hardware triggered strobe)
//                 1 1 0 = Mode 2 (rate generator, same as 010b)
//                 1 1 1 = Mode 3 (square wave generator, same as 011b)
// 0            BCD/Binary mode: 0 = 16-bit binary, 1 = four-digit BCD

impl Speaker {
    /// Create a new Speaker instance.
    pub const fn new() -> Self {
        Speaker {
            pit_ctrl_port: IoPort::new(SpeakerRegister::Control as u16),
            pit_data0_port: IoPort::new(SpeakerRegister::Data0 as u16),
            pit_data2_port: IoPort::new(SpeakerRegister::Data2 as u16),
            ppi_port: IoPort::new(SpeakerRegister::PPI as u16),
        }
    }

    /// Play a specific frequency for a given amount of time (milliseconds).
    pub fn play(&mut self, frequency: usize, duration: usize) {
        let counter = PIT_FREQUENCY / frequency;

        // Configure PIT counter 2 for mode 3 (square wave generator)
        // Control byte: 10 (counter 2), 11 (load LSB then MSB), 011 (mode 3), 0 (binary)
        // = 1011_0110 = 0xB6
        unsafe { self.pit_ctrl_port.outb(0xB6u8); }

        // Load the reload value (LSB then MSB)
        unsafe { self.pit_data2_port.outb((counter & 0xFF) as u8); } // Load channel 2 (low byte)
        unsafe { self.pit_data2_port.outb((counter >> 8)   as u8); } // Load channel 2 (high byte)

        self.on();
        pit::wait(duration);
        // self.off();
    }

    /// Turn on the speaker.
    /// The played tone is dependent on counter 2 of the PIT.
    pub fn on(&mut self) {
        let ppi_value = unsafe { self.ppi_port.inb() };
        // Set bits 0 and 1 to enable speaker
        unsafe { self.ppi_port.outb(ppi_value | 0x03u8); } // 0x03 == 0b00000011
    }

    /// Turn off the speaker.
    pub fn off(&mut self) {
        let ppi_value = unsafe { self.ppi_port.inb() };
        // Clear bits 0 and 1 to disable speaker
        unsafe { self.ppi_port.outb(ppi_value & 0xFCu8); } // 0xFC == 0b11111100
    }

}

/// Plays the Tetris theme using the PC speaker.
/// Kévin Rapaille, August 2013, https://gist.github.com/XeeX/6220067
pub fn tetris() {
    let mut speaker = SPEAKER.lock();

    speaker.play(658, 125);
    speaker.play(1320, 500);
    speaker.play(990, 250);
    speaker.play(1056, 250);
    speaker.play(1188, 250);
    speaker.play(1320, 125);
    speaker.play(1188, 125);
    speaker.play(1056, 250);
    speaker.play(990, 250);
    speaker.play(880, 500);
    speaker.play(880, 250);
    speaker.play(1056, 250);
    speaker.play(1320, 500);
    speaker.play(1188, 250);
    speaker.play(1056, 250);
    speaker.play(990, 750);
    speaker.play(1056, 250);
    speaker.play(1188, 500);
    speaker.play(1320, 500);
    speaker.play(1056, 500);
    speaker.play(880, 500);
    speaker.play(880, 500);
    pit::wait(250);
    speaker.play(1188, 500);
    speaker.play(1408, 250);
    speaker.play(1760, 500);
    speaker.play(1584, 250);
    speaker.play(1408, 250);
    speaker.play(1320, 750);
    speaker.play(1056, 250);
    speaker.play(1320, 500);
    speaker.play(1188, 250);
    speaker.play(1056, 250);
    speaker.play(990, 500);
    speaker.play(990, 250);
    speaker.play(1056, 250);
    speaker.play(1188, 500);
    speaker.play(1320, 500);
    speaker.play(1056, 500);
    speaker.play(880, 500);
    speaker.play(880, 500);
    pit::wait(500);
    speaker.play(1320, 500);
    speaker.play(990, 250);
    speaker.play(1056, 250);
    speaker.play(1188, 250);
    speaker.play(1320, 125);
    speaker.play(1188, 125);
    speaker.play(1056, 250);
    speaker.play(990, 250);
    speaker.play(880, 500);
    speaker.play(880, 250);
    speaker.play(1056, 250);
    speaker.play(1320, 500);
    speaker.play(1188, 250);
    speaker.play(1056, 250);
    speaker.play(990, 750);
    speaker.play(1056, 250);
    speaker.play(1188, 500);
    speaker.play(1320, 500);
    speaker.play(1056, 500);
    speaker.play(880, 500);
    speaker.play(880, 500);
    pit::wait(250);
    speaker.play(1188, 500);
    speaker.play(1408, 250);
    speaker.play(1760, 500);
    speaker.play(1584, 250);
    speaker.play(1408, 250);
    speaker.play(1320, 750);
    speaker.play(1056, 250);
    speaker.play(1320, 500);
    speaker.play(1188, 250);
    speaker.play(1056, 250);
    speaker.play(990, 500);
    speaker.play(990, 250);
    speaker.play(1056, 250);
    speaker.play(1188, 500);
    speaker.play(1320, 500);
    speaker.play(1056, 500);
    speaker.play(880, 500);
    speaker.play(880, 500);
    pit::wait(500);
    speaker.play(660, 1000);
    speaker.play(528, 1000);
    speaker.play(594, 1000);
    speaker.play(495, 1000);
    speaker.play(528, 1000);
    speaker.play(440, 1000);
    speaker.play(419, 1000);
    speaker.play(495, 1000);
    speaker.play(660, 1000);
    speaker.play(528, 1000);
    speaker.play(594, 1000);
    speaker.play(495, 1000);
    speaker.play(528, 500);
    speaker.play(660, 500);
    speaker.play(880, 1000);
    speaker.play(838, 2000);
    speaker.play(660, 1000);
    speaker.play(528, 1000);
    speaker.play(594, 1000);
    speaker.play(495, 1000);
    speaker.play(528, 1000);
    speaker.play(440, 1000);
    speaker.play(419, 1000);
    speaker.play(495, 1000);
    speaker.play(660, 1000);
    speaker.play(528, 1000);
    speaker.play(594, 1000);
    speaker.play(495, 1000);
    speaker.play(528, 500);
    speaker.play(660, 500);
    speaker.play(880, 1000);
    speaker.play(838, 2000);
    speaker.off();
}

/// Plays part of the song "Aerodynamic" by Daft Punk using the PC speaker.
/// https://www.kirrus.co.uk/2010/09/linux-beep-music
pub fn aerodynamic() {
    let mut speaker = SPEAKER.lock();

    speaker.play(587, 122);
    speaker.play(370, 122);
    speaker.play(493, 122);
    speaker.play(370, 122);
    speaker.play(587, 122);
    speaker.play(370, 122);
    speaker.play(493, 122);
    speaker.play(370, 122);
    speaker.play(587, 122);
    speaker.play(370, 122);
    speaker.play(493, 122);
    speaker.play(370, 122);
    speaker.play(587, 122);
    speaker.play(370, 122);
    speaker.play(493, 122);
    speaker.play(370, 122);
    speaker.play(587, 122);
    speaker.play(415, 122);
    speaker.play(493, 122);
    speaker.play(415, 122);
    speaker.play(587, 122);
    speaker.play(415, 122);
    speaker.play(493, 122);
    speaker.play(415, 122);
    speaker.play(587, 122);
    speaker.play(415, 122);
    speaker.play(493, 122);
    speaker.play(415, 122);
    speaker.play(587, 122);
    speaker.play(415, 122);
    speaker.play(493, 122);
    speaker.play(415, 122);
    speaker.play(784, 122);
    speaker.play(493, 122);
    speaker.play(659, 122);
    speaker.play(493, 122);
    speaker.play(784, 122);
    speaker.play(493, 122);
    speaker.play(659, 122);
    speaker.play(493, 122);
    speaker.play(784, 122);
    speaker.play(493, 122);
    speaker.play(659, 122);
    speaker.play(493, 122);
    speaker.play(784, 122);
    speaker.play(493, 122);
    speaker.play(659, 122);
    speaker.play(493, 122);
    speaker.play(659, 122);
    speaker.play(440, 122);
    speaker.play(554, 122);
    speaker.play(440, 122);
    speaker.play(659, 122);
    speaker.play(440, 122);
    speaker.play(554, 122);
    speaker.play(440, 122);
    speaker.play(659, 122);
    speaker.play(440, 122);
    speaker.play(554, 122);
    speaker.play(440, 122);
    speaker.play(659, 122);
    speaker.play(440, 122);
    speaker.play(554, 122);
    speaker.play(440, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(587, 122);
    speaker.play(370, 122);
    speaker.play(493, 122);
    speaker.play(370, 122);
    speaker.play(587, 122);
    speaker.play(370, 122);
    speaker.play(493, 122);
    speaker.play(370, 122);
    speaker.play(587, 122);
    speaker.play(370, 122);
    speaker.play(493, 122);
    speaker.play(370, 122);
    speaker.play(587, 122);
    speaker.play(370, 122);
    speaker.play(493, 122);
    speaker.play(370, 122);
    speaker.play(587, 122);
    speaker.play(415, 122);
    speaker.play(493, 122);
    speaker.play(415, 122);
    speaker.play(587, 122);
    speaker.play(415, 122);
    speaker.play(493, 122);
    speaker.play(415, 122);
    speaker.play(587, 122);
    speaker.play(415, 122);
    speaker.play(493, 122);
    speaker.play(415, 122);
    speaker.play(587, 122);
    speaker.play(415, 122);
    speaker.play(493, 122);
    speaker.play(415, 122);
    speaker.play(784, 122);
    speaker.play(493, 122);
    speaker.play(659, 122);
    speaker.play(493, 122);
    speaker.play(784, 122);
    speaker.play(493, 122);
    speaker.play(659, 122);
    speaker.play(493, 122);
    speaker.play(784, 122);
    speaker.play(493, 122);
    speaker.play(659, 122);
    speaker.play(493, 122);
    speaker.play(784, 122);
    speaker.play(493, 122);
    speaker.play(659, 122);
    speaker.play(493, 122);
    speaker.play(659, 122);
    speaker.play(440, 122);
    speaker.play(554, 122);
    speaker.play(440, 122);
    speaker.play(659, 122);
    speaker.play(440, 122);
    speaker.play(554, 122);
    speaker.play(440, 122);
    speaker.play(659, 122);
    speaker.play(440, 122);
    speaker.play(554, 122);
    speaker.play(440, 122);
    speaker.play(659, 122);
    speaker.play(440, 122);
    speaker.play(554, 122);
    speaker.play(587, 122);
    speaker.play(370, 122);
    speaker.play(493, 122);
    speaker.play(370, 122);
    speaker.play(587, 122);
    speaker.play(370, 122);
    speaker.play(493, 122);
    speaker.play(370, 122);
    speaker.play(587, 122);
    speaker.play(370, 122);
    speaker.play(493, 122);
    speaker.play(370, 122);
    speaker.play(587, 122);
    speaker.play(370, 122);
    speaker.play(493, 122);
    speaker.play(370, 122);
    speaker.play(587, 122);
    speaker.play(415, 122);
    speaker.play(493, 122);
    speaker.play(415, 122);
    speaker.play(587, 122);
    speaker.play(415, 122);
    speaker.play(493, 122);
    speaker.play(415, 122);
    speaker.play(587, 122);
    speaker.play(415, 122);
    speaker.play(493, 122);
    speaker.play(415, 122);
    speaker.play(587, 122);
    speaker.play(415, 122);
    speaker.play(493, 122);
    speaker.play(415, 122);
    speaker.play(784, 122);
    speaker.play(493, 122);
    speaker.play(659, 122);
    speaker.play(493, 122);
    speaker.play(784, 122);
    speaker.play(493, 122);
    speaker.play(659, 122);
    speaker.play(493, 122);
    speaker.play(784, 122);
    speaker.play(493, 122);
    speaker.play(659, 122);
    speaker.play(493, 122);
    speaker.play(784, 122);
    speaker.play(493, 122);
    speaker.play(659, 122);
    speaker.play(493, 122);
    speaker.play(659, 122);
    speaker.play(440, 122);
    speaker.play(554, 122);
    speaker.play(440, 122);
    speaker.play(659, 122);
    speaker.play(440, 122);
    speaker.play(554, 122);
    speaker.play(440, 122);
    speaker.play(659, 122);
    speaker.play(440, 122);
    speaker.play(554, 122);
    speaker.play(440, 122);
    speaker.play(659, 122);
    speaker.play(440, 122);
    speaker.play(554, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(740, 122);
    speaker.play(987, 122);
    speaker.play(740, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1174, 122);
    speaker.play(830, 122);
    speaker.play(987, 122);
    speaker.play(830, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1568, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(987, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.play(1318, 122);
    speaker.play(880, 122);
    speaker.play(1108, 122);
    speaker.play(880, 122);
    speaker.off();
}
