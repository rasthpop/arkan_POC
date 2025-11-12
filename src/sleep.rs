use core::sync::atomic::{AtomicBool, Ordering};

use cortex_m::asm::wfi;
use cortex_m::peripheral::NVIC;
use rp_pico::hal::fugit::ExtU32;
use rp_pico::hal::pac::{self, interrupt, Interrupt};
use rp_pico::hal::timer::{Alarm, Timer};

// Flag raised from the timer interrupt when the alarm expires.
static ALARM0_FIRED: AtomicBool = AtomicBool::new(false);

#[interrupt]
fn TIMER_IRQ_0() {
    let timer = unsafe { &*pac::TIMER::ptr() };

    if timer.intr().read().alarm_0().bit_is_set() {
        timer.intr().write(|w| w.alarm_0().bit(true));
        ALARM0_FIRED.store(true, Ordering::SeqCst);
    }
}

/// Timed sleep using Timer Alarm0 + NVIC + WFI.
pub fn sleep_ms(timer: &mut Timer, ms: u32) {
    let mut alarm0 = timer.alarm_0().unwrap();

    ALARM0_FIRED.store(false, Ordering::SeqCst);
    alarm0.clear_interrupt();

    // Ensure the CPU will wake when the alarm fires.
    cortex_m::peripheral::NVIC::unpend(Interrupt::TIMER_IRQ_0);
    unsafe { NVIC::unmask(Interrupt::TIMER_IRQ_0); }
    alarm0.enable_interrupt();

    let _ = alarm0.schedule(ms.millis());

    while !ALARM0_FIRED.load(Ordering::SeqCst) {
        wfi();
    }

    alarm0.disable_interrupt();
    alarm0.clear_interrupt();
    unsafe { NVIC::mask(Interrupt::TIMER_IRQ_0); }
}

/// Disable UART0 transmit/receive paths to reduce power before sleep.
pub fn disable_uart0() {
    unsafe {
        let uart0 = &*pac::UART0::ptr();
        uart0
            .uartcr()
            .modify(|_, w| w.uarten().clear_bit().txe().clear_bit().rxe().clear_bit());
    }
}

/// Re-enable UART0 after waking from sleep.
pub fn enable_uart0() {
    unsafe {
        let uart0 = &*pac::UART0::ptr();
        uart0
            .uartcr()
            .modify(|_, w| w.uarten().set_bit().txe().set_bit().rxe().set_bit());
    }
}