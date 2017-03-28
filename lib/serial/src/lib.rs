#![feature(const_fn)]
#![no_std]

extern crate x86;

extern crate spin;

use x86::io::{inb, outb};

use spin::Mutex;

pub struct SerialWriter {
    mode: SerialMode,
}

pub static SERIAL_WRITER: Mutex<SerialWriter> = Mutex::new(SerialWriter {
    mode: SerialMode::UnInit,
});

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let mut writer = $crate::SERIAL_WRITER.lock();
        if let Err(err) = writer.write_fmt(format_args!($($arg)*)) {
            panic!("{}", err);
        }
    });
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

/// I/O port base address for the first serial port.
const IO_BASE: u16 = 0x3f8;

/// DLAB=0 registers. @{
const RBR_REG: u16 = (IO_BASE + 0);  ///< Receiver Buffer Reg. (read-only).
const THR_REG: u16 = (IO_BASE + 0);  ///< Transmitter Holding Reg. (write-only).
const IER_REG: u16 = (IO_BASE + 1);  ///< Interrupt Enable Reg..
/* @} */

/// DLAB=1 registers. @{
const LS_REG: u16 = (IO_BASE + 0);   ///< Divisor Latch (LSB).
const MS_REG: u16 = (IO_BASE + 1);   ///< Divisor Latch (MSB).
/// @}

/// DLAB-insensitive registers. @{
const IIR_REG: u16 = (IO_BASE + 2);  ///< Interrupt Identification Reg. (read-only)
const FCR_REG: u16 = (IO_BASE + 2);  ///< FIFO Control Reg. (write-only).
const LCR_REG: u16 = (IO_BASE + 3);  ///< Line Control Register.
const MCR_REG: u16 = (IO_BASE + 4);  ///< MODEM Control Register.
const LSR_REG: u16 = (IO_BASE + 5);  ///< Line Status Register (read-only).
/// @}

/// Interrupt Enable Register bits. @{
const IER_RECV: u8 = 0x01;          /// Interrupt when data received.
const IER_XMIT: u8 = 0x02;          /// Interrupt when transmit finishes.
/// @}

/// Line Control Register bits. @{
const LCR_N81: u8 = 0x03;           ///< No parity, 8 data bits, 1 stop bit.
const LCR_DLAB: u8 = 0x80;          ///< Divisor Latch Access Bit (DLAB).
/// @}

/// MODEM Control Register. @{
const MCR_OUT2: u8 = 0x08;          ///< Output line 2.
/// @}

/// Line Status Register. @{
const LSR_DR: u8 = 0x01;            ///< Data Ready: received data byte is in RBR.
const LSR_THRE: u8 = 0x20;          ///< THR Empty.
/// @}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SerialMode {
    UnInit=0isize,
    Poll,
    Queue,
}

use core::fmt::Write;
impl Write for SerialWriter {
    fn write_str(&mut self, string:&str) -> ::core::fmt::Result {
        self.print_str(string);
        Ok(())
    }
}

impl SerialWriter {
    pub unsafe fn new() -> SerialWriter {
        let mut result = SerialWriter {
            mode: SerialMode::UnInit,
        };
        result.init_poll();
        result
    }

    /// Initializes the serial port device for polling mode.  Polling mode
    /// busy-waits for the serial port to become free before writing to it.  It's
    /// slow, but until interrupts have been initialized it's all we can do.
    fn init_poll(&mut self) {
        unsafe {
            assert!(self.mode == SerialMode::UnInit);
            outb(IER_REG, 0);                   /* Turn off all interrupts. */
            outb(FCR_REG, 0);                   /* Disable FIFO. */
        }
        self.set_serial(9600);                       /* 9.6 kbps, N-8-1. */
        //outb(MCR_REG, MCR_OUT2);                /* Required to enable interrupts. */
        //intq_init(&txq);
        self.mode = SerialMode::Poll;
    }

    /// Configures the serial port for BPS bits per second.
    fn set_serial(&self, bps:u32) {
        let base_rate:u32 = 1843200 / 16;         /* Base rate of 16550A, in Hz. */
        let divisor:u16 = (base_rate / bps) as u16;   /* Clock rate divisor. */

        assert!(bps >= 300 && bps <= 115200);

        unsafe {
            /* Enable DLAB. */
            outb(LCR_REG, LCR_N81 | LCR_DLAB);

            /* Set data rate. */
            outb(LS_REG, (divisor & 0xff) as u8);
            outb(MS_REG, (divisor >> 8) as u8);

            /* Reset DLAB. */
            outb(LCR_REG, LCR_N81);
        }
    }

    pub fn print_str(&mut self, string:&str) {
        for byte in string.bytes() {
            self.putc(byte);
        }
    }

    /// Sends BYTE to the serial port.
    pub fn putc(&mut self, byte:u8) {
        //enum intr_level old_level = intr_disable();

        if self.mode != SerialMode::Queue {
            /* If we're not set up for interrupt-driven I/O yet,
               use dumb polling to transmit a byte. */
            if self.mode == SerialMode::UnInit {
                self.init_poll();
            }
            self.putc_poll(byte);
        }
        else {
            /* Otherwise, queue a byte and update the interrupt enable register. */
            //if (old_level == INTR_OFF && intq_full(&txq)) {
                /* Interrupts are off and the transmit queue is full.
                   If we wanted to wait for the queue to empty,
                   we'd have to reenable interrupts.
                   That's impolite, so we'll send a character via
                   polling instead. */
                //putc_poll(intq_getc (&txq));
            //}

            //intq_putc(&txq, byte);
            //write_ier();
            unimplemented!();
        }

        //intr_set_level(old_level);
    }

    /// Polls the serial port until it's ready, and then transmits BYTE.
    fn putc_poll(&self, byte:u8) {
        //assert!(intr_get_level() == InterruptLevel::Off);

        unsafe {
            while (inb(LSR_REG) & LSR_THRE) == 0 {
                continue;
            }
            outb(THR_REG, byte);
        }
    }
}

