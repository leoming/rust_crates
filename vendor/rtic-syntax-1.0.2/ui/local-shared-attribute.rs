#![no_main]

#[mock::app]
mod app {
    #[task(local = [
        #[test]
        a: u32 = 0, // Ok
        #[test]
        b, // Error
    ])]
    fn foo(_: foo::Context) {

    }
}
