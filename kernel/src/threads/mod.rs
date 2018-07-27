use ::core::sync::atomic;

use ::interrupts::InterruptGuard;

use ::atomic_ring_buffer::AtomicRingBuffer;
static ALL_THREADS: AtomicRingBuffer<*mut Thread, [*mut Thread; 6]> =
    AtomicRingBuffer::new([0 as *mut Thread; 6]);

pub struct Tid(u64);

#[repr(u64)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Status {
    Ready,
    Running,
    Dying,
}

pub struct AtomicStatus {
    status: atomic::AtomicUsize,
}

impl AtomicStatus {
    pub fn load(&self, order: atomic::Ordering) -> Status {
        unsafe {
            ::core::mem::transmute(self.status.load(order))
        }
    }

    pub fn store(&self, value: usize, order: atomic::Ordering) {
        unsafe {
            self.status.store(::core::mem::transmute(value), order)
        }
    }
}

const MAX_TICKS: u64 = 0x5;
const DATA_SIZE: usize = 0x1000 - (6 * 0x8);
#[repr(C)]
pub struct Thread {
    tid: Tid,
    ticks: u64,
    status: Status,
    is_idle: bool,
    stack: *mut u8,
    magic: u64,
    data: [u8; DATA_SIZE],
}

pub const THREAD_MAGIC: u64 = 0xDEADBEEF;

pub fn current_thread_ptr() -> *mut Thread {
    let stack: usize;
    unsafe {
        asm!("
            mov %rsp, $0
            ": "=r"(stack) );
    }
    (stack & 0xFFFF_FFFF_FFFF_F000) as *mut Thread
}

pub fn current_thread(_: &InterruptGuard) -> &mut Thread {
    unsafe {
        ::core::mem::transmute(current_thread_ptr())
    }
}

macro_rules! push_all_registers {
    ( ) => {
        unsafe {
            asm!("
                push %r15
                push %r14
                push %r13
                push %r12
                push %r11
                push %r10
                push %r9
                push %r8
                push %rbp
                push %rdi
                push %rsi
                push %rdx
                push %rcx
                push %rbx
                ");
            // Still need to push simd registers
        }
    }
}

macro_rules! pop_all_registers {
    ( ) => {
        unsafe {
            asm!("
                pop %rbx
                pop %rcx
                pop %rdx
                pop %rsi
                pop %rdi
                pop %rbp
                pop %r8
                pop %r9
                pop %r10
                pop %r11
                pop %r12
                pop %r13
                pop %r14
                pop %r15
                ");
        }
    }
}

#[repr(C)]
struct ThreadRegisters {
    rax: usize,
    rbx: usize,
    rcx: usize,
    rdx: usize,
    rsi: usize,
    rdi: usize,
    rbp: usize,
    rflags: usize,
    r8: usize,
    r9: usize,
    r10: usize,
    r11: usize,
    r12: usize,
    r13: usize,
    r14: usize,
    r15: usize,
}

impl Thread {
    /*
    pub fn new(fn_ptr: fn()) -> Thread {
        let mut data = [0u8; DATA_SIZE];
        unsafe {
            let pointer: *mut u8 = (&mut data[DATA_SIZE - 0x10]) as *mut u8;
            *(pointer as *mut u64) = fn_ptr as u64;
        }
        Thread {
            tid: Tid(0),
            ticks: MAX_TICKS,
            status: Status::Ready,
            stack: 0x1000 - 0x10 as *mut u8,
            magic: THREAD_MAGIC,
            data,
        }
    }
    */

    pub fn new_in_place(ptr: *mut Thread, fn_ptr: fn()) {
        Thread::new_in_place_no_insert(ptr, false, fn_ptr);

        ALL_THREADS.enqueue(|thread_ptr| { *thread_ptr = ptr; });
    }

    pub fn new_in_place_no_insert(ptr: *mut Thread, is_idle: bool, fn_ptr: fn()) {
        unsafe {
            (*ptr).tid = Tid(0);
            (*ptr).ticks = MAX_TICKS;
            (*ptr).status = Status::Ready;
            (*ptr).stack = (ptr as *mut u8).offset(0x1000 - 0x10);
            (*ptr).magic = THREAD_MAGIC;
            ::rlibc::memset(&mut (*ptr).data[0] as *mut u8, 0i32, DATA_SIZE);
            let pointer: *mut u8 = (&mut (*ptr).data[DATA_SIZE - 0x28]) as *mut u8;
            *(pointer as *mut u64) = fn_ptr as u64;
            
            let pointer: *mut u8 = (&mut (*ptr).data[DATA_SIZE - 0x10]) as *mut u8;
            *(pointer as *mut u64) = kernel_thread_entry as u64;

            let pointer: *mut u8 = (&mut (*ptr).data[DATA_SIZE - 0x8]) as *mut u8;
            *(pointer as *mut u64) = 0 as u64;
        }
    }

    /*pub fn new(fn_ptr: &fn()) -> Thread {
        let frame = ::falloc::FRAME_ALLOCATOR.get_frame();
        let page = ::palloc::PAGE_ALLOCATOR.get_page();

        page_table.insert_page(frame, page);

        let thread: Thread = page;
        thread
    }*/
    pub fn start_first_thread(&mut self, single_execution: ::interrupts::SingleExecution) -> ! {
        self.status = Status::Running;
        ::core::mem::drop(single_execution);
        unsafe {
            asm!("
                mov $0, %rsp
                mov $$0, %rbp
                mov $$0, %rax
                ret
                " : : "r"(self.stack));
            ::core::intrinsics::unreachable();
        }
    }

    pub fn switch_to_thread(&mut self) -> *mut Thread {
        let mut last_thread: *mut Thread;
        push_all_registers!();
        {
            let thread: *mut Thread = current_thread_ptr();

            // Push return address
            // Save stack pointer
            // Switch to new thread
            unsafe {
                asm!("
                    lea 0xd(%rip), %r15
                    push %r15
                    mov %rsp, ($2)
                    mov ($1), %rsp
                    mov $0, $3
                    ret
                    new_thread:nop
                    " : "={rax}"(last_thread) : "r"(&self.stack), "r"(&(*thread).stack), "r"(thread) );
            }
        }
        pop_all_registers!();
        last_thread
    }

    pub fn tick(&mut self) -> u64 {
        self.ticks -= 1;
        self.ticks
    }

    pub fn reset_ticks(&mut self) {
        self.ticks = MAX_TICKS;
    }

    pub fn exit(&mut self) -> ! {
        self.status = Status::Dying;

        let interrupt_guard = ::interrupts::disable();
        self.schedule(&interrupt_guard, Status::Ready);

        unreachable!();
    }

    pub fn schedule(&mut self, interrupt_guard: &InterruptGuard, status: Status) {
        assert!(self.magic == THREAD_MAGIC);
        assert!(self.status != Status::Ready);
        let next_thread = {
            let mut next_thread = None;
            ALL_THREADS.dequeue(|thread| { next_thread = Some(*thread) });
            next_thread
        };

        let status = self.status;
        let self_ptr = self as *mut Thread;
        let next_thread = next_thread.unwrap_or_else(|| {
            if status == Status::Running {
                self_ptr
            } else {
                // THIS IS A COPY. BEWARE
                *::per_cpu::retrieve_per_cpu(&interrupt_guard).get_idle_thread()
            }
        });

        if next_thread != self_ptr {
            unsafe {
                // TODO make switch_to_thread consume and return an interrupt_guard
                let last_thread = (*next_thread).switch_to_thread();
                (*last_thread).finish_switching();
            }
        }
        self.reset_ticks();
        self.status = Status::Running;
    }

    // TODO should this take in self or thread pointer?
    pub unsafe fn finish_switching(&mut self) {
        

        if self.status == Status::Dying {
            unsafe {
                use ::core::fmt::Write;
                let mut writer = ::serial::SerialWriter::new_init();
                if let Err(err) = writer.write_fmt(format_args!("exited thread {:x}\n", self as *const Thread as u64)) {
                    panic!("{}", err);
                }
            }
        }

        if self.status == Status::Running {
            self.status = Status::Ready;
            if !self.is_idle {
                ALL_THREADS.enqueue_spin(|thread_ptr| {
                    *thread_ptr = self as *mut Thread;
                });
            }
        }

        // if thread is exiting destroy it
    }
}

pub extern "cdecl" fn kernel_thread_entry(last_thread: *mut Thread, fn_ptr: fn()) -> ! {
    unsafe {
        asm!("
            mov -0x18(%rbp), %rsi
            mov %rsi, %rbx
            mov %rax, %rdi
            ");
        if last_thread != 0 as *mut Thread {
            (*last_thread).finish_switching();
        }
        let thread = current_thread_ptr();
        (*thread).status = Status::Running;
        ::x86::shared::irq::enable();
        fn_ptr();
    }
    unsafe {
        let thread = current_thread_ptr();
        (*thread).exit();
    }
}

pub fn idle_thread() {
    loop {
        /* Let someone else run in case an interrupt happened that
         * unblocked another thread without scheduling. */
        //let interrupt_guard = intr_disable();
        //thread_block(&interrupt_guard);
        //mem::forget(interrupt_guard); // prevent the sti from happening until we halt

        /* Re-enable interrupts and wait for the next one.

           The `sti' instruction disables interrupts until the completion of
           the next instruction, so these two instructions are executed
           atomically.  This atomicity is important; otherwise, an interrupt
           could be handled between re-enabling interrupts and waiting for the
           next one to occur, wasting as much as one clock tick worth of time.

           See [IA32-v2a] "HLT", [IA32-v2b] "STI", and [IA32-v3a]
           7.11.1 "HLT Instruction". */
        unsafe {
        //asm volatile ("sti; hlt" : : : "memory");
            asm!("hlt");
        }
    }
}
