use std::{thread::sleep, time::Duration};

static TICK_RATE: f32 = 10.;

fn main() {
    loop {
        println!("Tick");
        sleep(Duration::from_secs_f32(1. / TICK_RATE));
    }
}
