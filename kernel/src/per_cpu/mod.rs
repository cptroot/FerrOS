use ::core::ops::Deref;

use x86::shared::msr::{wrmsr, rdmsr, IA32_TSC_AUX};

use ::threads::Thread;
use ::initialize_once::InitializeOnce;

use ::interrupts::{SingleExecution, InterruptGuard};

use ::palloc::{get_contiguous_array, PallocArray};

pub struct ProcessorId {
    id: u64,
}

impl ProcessorId {
    pub const fn new(id: u64) -> Self {
        ProcessorId {
            id,
        }
    }
}

pub unsafe fn write_processor_id(id: ProcessorId) {
    wrmsr(IA32_TSC_AUX, id.id);
}

pub fn read_processor_id() -> ProcessorId {
    unsafe {
        ProcessorId::new(rdmsr(IA32_TSC_AUX))
    }
}

static PER_CPU_PTR: GlobalPerCpu = GlobalPerCpu::new();

struct GlobalPerCpu {
    ptr: InitializeOnce<PallocArray<PerCpuData>>,
}

impl GlobalPerCpu {
    pub const fn new() -> Self {
        GlobalPerCpu {
            ptr: InitializeOnce::new(),
        }
    }

    pub fn initialize(&self, single_execution: &SingleExecution, ptr: ::palloc::PallocArray<PerCpuData>) {
        self.ptr.initialize(&single_execution, ptr);
    }

    pub fn get_per_cpu<'interrupts, 'this: 'interrupts>(&'this self, _: &'interrupts InterruptGuard) -> &'interrupts PerCpuData {
        // retrieve per_cpu block
        let per_cpu_block = self.ptr.deref();
        // offset and convert to mut pointer
        let processor_id = read_processor_id().id;
        &per_cpu_block[processor_id as usize]
    }
}

pub struct PerCpuData {
    idle_thread: InitializeOnce<*mut Thread>,
}

// Can only be called once
pub unsafe fn initialize_per_cpu(single_execution: &SingleExecution, num_cores: usize) {
    //palloc per_cpu block with blank data
    let per_cpu_block = get_contiguous_array(num_cores, PerCpuData::new);
    //initialize per_cpu
    PER_CPU_PTR.initialize(single_execution, per_cpu_block);
}

// Idea, require this function to be called inside of an interrupt context
// Tie the lifetime of a mutable borrow to the lifetime of the interrupt context
/// The interrupt guard ensures that interrupts are off when this function is called
pub fn retrieve_per_cpu(guard: &InterruptGuard) -> &PerCpuData {
    PER_CPU_PTR.get_per_cpu(guard)
}


impl PerCpuData {
    pub fn new() -> Self {
        PerCpuData {
            idle_thread: InitializeOnce::new(),
        }
    }
    pub fn set_idle_thread(&self, single: &SingleExecution, idle_thread: *mut Thread) {
        // remove thread from all_threads list
        // set idle thread
        self.idle_thread.initialize(single, idle_thread);
    }
    pub fn get_idle_thread(&self) -> &*mut Thread {
        self.idle_thread.deref()
    }
}
