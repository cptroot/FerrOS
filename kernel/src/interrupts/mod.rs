

pub struct SingleExecution {
    _private_value: (),
}

impl SingleExecution {
    pub unsafe fn new() -> Self {
        SingleExecution {
            _private_value: (),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InterruptState {
    Enabled,
    Disabled,
}

pub struct InterruptGuard {
    previous_interrupts: InterruptState,
}

impl InterruptGuard {
    pub unsafe fn new(previous_interrupts: InterruptState) -> InterruptGuard {
        InterruptGuard {
            previous_interrupts,
        }
    }
}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        if self.previous_interrupts == InterruptState::Enabled {
            unsafe {
                ::x86::shared::irq::enable();
            }
        }
    }
}

const FLAG_IF: usize = 0x00000200;
pub fn get_interrupts_level() -> InterruptState {
    let flags: usize;

    /* Push the flags register on the processor stack, then pop the
     * value off the stack into `flags'.  See [IA32-v2b] "PUSHF"
     * and "POP" and [IA32-v3a] 5.8.1 "Masking Maskable Hardware
     * Interrupts". */
    unsafe {
        asm!("pushfq; popq $0" : "=r" (flags) : : : "volatile");
    }

    if flags & FLAG_IF == FLAG_IF {
        InterruptState::Enabled
    } else {
        InterruptState::Disabled
    }
}

pub fn disable() -> InterruptGuard {
    let previous_interrupts = get_interrupts_level();
    unsafe {
        ::x86::shared::irq::disable();
    }
    InterruptGuard {
        previous_interrupts: previous_interrupts,  
    }
}
