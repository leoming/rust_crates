#![no_main]

#[mock::app]
mod app {
    #[monotonic(binds = Tim1)]
    type Fast1 = hal::Tim1Monotonic;

    #[monotonic(binds = Tim1)]
    type Fast2 = hal::Tim2Monotonic;
}
