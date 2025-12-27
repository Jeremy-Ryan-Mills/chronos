//! Global Descriptor Table (GDT) and Task State Segment (TSS) setup.
//!
//! On x86_64, we still use a GDT for a few key things in long mode:
//! - setting the kernel code segment selector
//! - loading a TSS, which provides Interrupt Stack Table (IST) entries
//!
//! The IST is especially useful for handling faults like a double fault on a
//! known-good stack (e.g., if the normal kernel stack is corrupted/overflowed).

use lazy_static::lazy_static;
use x86_64::VirtAddr;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

/// IST slot used for the double fault handler.
///
/// This index must match what the IDT double-fault entry is configured to use.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

/// Segment selectors we need after loading the GDT.
///
/// In long mode the segmentation model is mostly “flat”, but the CPU still uses
/// selectors for things like CS and the TSS descriptor.
struct Selectors {
    /// Kernel code segment selector.
    code_selector: SegmentSelector,
    /// TSS segment selector (points at the TSS descriptor in the GDT).
    tss_selector: SegmentSelector,
}

lazy_static! {
    /// Task State Segment for this CPU.
    ///
    /// We primarily use the TSS to provide an Interrupt Stack Table entry for
    /// double faults so that they run on a dedicated stack.
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // Provide a separate stack for double faults. If a double fault occurs
        // because the normal stack is broken, switching stacks here can be the
        // difference between a useful panic and an immediate reset.
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            // Use the end of the stack as the initial stack pointer (stacks grow down).
            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        tss
    };
}

lazy_static! {
    /// The GDT plus the selectors for the entries we care about.
    ///
    /// We install:
    /// - a kernel code segment descriptor
    /// - a TSS descriptor pointing to [`TSS`]
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector  = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

/// Load the GDT and activate the TSS.
///
/// This should be called early during boot, before installing IDT entries that
/// rely on IST stacks (like the double-fault handler).
pub fn init() {
    use x86_64::instructions::segmentation::{CS, Segment};
    use x86_64::instructions::tables::load_tss;

    // Load the GDT itself.
    GDT.0.load();

    // Update CS and load the Task Register (TR) with the TSS selector.
    // These operations are privileged and must be done in an unsafe block.
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
