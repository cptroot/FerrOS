
pub struct Tid(u64);

pub enum Status {
    Ready,
    Running,
}

const MAX_TICKS: u64 = 0x5;
const DATA_SIZE: usize = 0xFD8;
#[repr(C)]
pub struct Thread {
    tid: Tid,
    ticks: u64,
    status: Status,
    stack: *mut u8,
    magic: u64,
    data: [u8; DATA_SIZE],
}

pub const THREAD_MAGIC: u64 = 0xDEADBEEF;

pub fn current_thread() -> *mut Thread {
    let stack: usize;
    unsafe {
        asm!("
            mov %rsp, $0
            ": "=r"(stack) );
    }
    (stack & 0xFFFF_FFFF_FFFF_F000) as *mut Thread
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
                pushfq
                push %rbp
                push %rdi
                push %rsi
                push %rdx
                push %rcx
                push %rbx
                push %rax
                ");
            // Still need to push simd registers
        }
    }
}

macro_rules! pop_all_registers {
    ( ) => {
        unsafe {
            asm!("
                pop %rax
                pop %rbx
                pop %rcx
                pop %rdx
                pop %rsi
                pop %rdi
                pop %rbp
                popfq
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
    pub fn start_first_thread(&mut self) -> ! {
        unsafe {
            asm!("
                mov $0, %rsp
                mov $$0, %rbp
                ret
                " : : "r"(self.stack));
            ::core::intrinsics::unreachable();
        }
    }

    pub fn switch_to_thread(&mut self) {
        push_all_registers!();
        {
            let thread: *mut Thread = current_thread();

            // Push return address
            // Save stack pointer
            // Switch to new thread
            unsafe {
                asm!("
                    lea 0x9(%rip), %r15
                    push %r15
                    mov %rsp, ($1)
                    mov ($0), %rsp
                    ret
                    new_thread:nop
                    " :  : "r"(&self.stack), "r"(&(*thread).stack) );
            }
        }
        pop_all_registers!();
    }

    pub fn tick(&mut self) -> u64 {
        self.ticks -= 1;
        self.ticks
    }

    pub fn reset_ticks(&mut self) {
        self.ticks = MAX_TICKS;
    }

    pub fn exit(&mut self) -> ! {
        loop {}
    }
}

pub extern "cdecl" fn kernel_thread_entry(fn_ptr: fn()) -> ! {
    unsafe {
        asm!("
            mov -0x18(%rbp), %rdi
            mov %rdi, %rbx
            ");
        ::x86::shared::irq::enable();
    }
    fn_ptr();
    unsafe {
        let thread = current_thread();
        (*thread).exit();
     }
}
