#![no_main]

#[mock::app]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        l1: u32,
        l2: u32,
    }

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

    // l2 ok
    #[idle(local = [l2])]
    fn idle(cx: idle::Context) -> ! {}

    // l1 rejected (not local)
    #[task(priority = 1, local = [l1])]
    fn uart0(cx: uart0::Context) {}

    // l1 rejected (not lock_free)
    #[task(priority = 2, local = [l1])]
    fn uart1(cx: uart1::Context) {}
}
