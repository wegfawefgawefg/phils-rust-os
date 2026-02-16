use core::sync::atomic::{AtomicU64, Ordering};

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{gdt, vga_text_mode_terminal::CURSOR_TOGGLE_FLAG};
use lazy_static::lazy_static;

use pic8259::ChainedPics;
use spin;

pub static TIMER_TICKS: AtomicU64 = AtomicU64::new(0);

////////////////    IDT    ////////////////
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX); // new
        }
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
}

pub fn init_pit(target_hz: u32) {
    use x86_64::instructions::port::Port;

    const PIT_BASE_HZ: u32 = 1_193_182;
    let hz = target_hz.max(1);
    let divisor: u16 = (PIT_BASE_HZ / hz).clamp(1, u16::MAX as u32) as u16;

    unsafe {
        let mut command = Port::new(0x43);
        let mut channel0 = Port::new(0x40);

        // Channel 0, lobyte/hibyte, mode 3 (square wave), binary counter.
        command.write(0x36u8);
        channel0.write((divisor & 0x00FF) as u8);
        channel0.write((divisor >> 8) as u8);
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    let _ = stack_frame;
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT\n{:#?}\n\n~~~EXECUTION HALTED~~~",
        stack_frame
    );
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TIMER_TICKS.fetch_add(1, Ordering::Relaxed);
    let current_state = CURSOR_TOGGLE_FLAG.load(Ordering::SeqCst);
    CURSOR_TOGGLE_FLAG.store(!current_state, Ordering::SeqCst);

    // Acknowledge the interrupt
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

pub fn timer_ticks() -> u64 {
    TIMER_TICKS.load(Ordering::Relaxed)
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore)
        );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        let _ = keyboard.process_keyevent(key_event);
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

////////////////    PIC    /////////////////////
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
}
//////////////// TRIGGER FAULTS ////////////////
pub fn trigger_page_fault() {
    unsafe {
        *(0xdeafdeef as *mut u8) = 42;
    };
}

#[allow(unconditional_recursion)]
pub fn overflow_stack() {
    overflow_stack();
}
