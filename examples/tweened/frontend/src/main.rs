use std::{sync::Arc, time::Instant};
use zoon::{println, *};

const ANIMATION_TIMEOUT: u32 = 1_000 / 60;

struct Easing;
impl Easing {
    fn cubic_out(t: f64) -> f64 {
        let f = t - 1.0;
        f * f * f + 1.0
    }
}

struct Interpolate;
impl Interpolate {
    fn linear(a: f64, b: f64) -> impl Fn(f64) -> f64 {
        let delta = b - a;
        move |t| a + t * delta
    }
}

struct Tweened;
impl Tweened {
    /// Creates and returns a mutable that listens for a new target value.
    /// After the signal fires, we start the interpolation which modifies another
    /// mutable for which we returns it's signal.
    fn new(value: f64, duration: f64, easing: impl Fn(f64) -> f64 + 'static) -> (Mutable<f64>, MutableSignal<f64>) {
        let (mutable, inner_signal) = Mutable::new_and_signal(value);
        let (inner_mutable, signal) = Mutable::new_and_signal(value);

        let timer: Mutable<Option<Timer>> = Mutable::new(None);
        let target_value = Mutable::new(value);
        let easing = Arc::new(easing);

        Task::start(inner_signal.for_each_sync(clone!((inner_mutable) move |value| {
            if timer.lock_ref().is_some() {
                drop(timer.lock_ref().as_ref());
                *timer.lock_mut() = None;
            }


            let interpolate = Interpolate::linear(target_value.get(), value);
            let now = || js_sys::Date::now();
            // FIXME: tried to use performance but somehow didn't work
            // let now = || web_sys::window().unwrap().performance().unwrap().now();
            let start = now();
            *target_value.lock_mut() = value;
            *timer.lock_mut() = Some(Timer::new(ANIMATION_TIMEOUT, clone!((inner_mutable, timer, easing, target_value) move || {
                let elapsed = (now() - start) as f64;

                if elapsed > duration {
                    inner_mutable.set(target_value.get());
                    drop(timer.lock_ref().as_ref());
                    *timer.lock_mut() = None;
                }

                inner_mutable.set((interpolate)((easing)(elapsed / duration)));
                println!("{}", inner_mutable.get());
            })));
        })));

        (mutable, signal)
    }
}

fn root() -> impl Element {
    // let (radius, radius_signal) = Tweened::new(1., 1000., &Easing::cubic_out);
    // let (cx, cx_signal) = Tweened::new(50., 1000., &Easing::cubic_out);
    // let (cy, cy_signal) = Tweened::new(50., 1000., &Easing::cubic_out);

    let speed = Duration::seconds(1);
    let radius = Oscillator::with_speed(speed);
    let cx = Oscillator::with_speed(speed);
    let cy = Oscillator::with_speed(speed);

    let circle = RawSvgEl::new("svg")
        .attr("width", "100%")
        .attr("height", "100%")
        .attr("viewbox", "0 0 300 300")
        .child(
            RawSvgEl::new("circle")
                .attr_signal("cx", cx.signal().map(ease::linear_unit(0, 1000)))
                .attr_signal("cy", cy.signal().map(ease::linear_unit(0, 600)))
                .attr("fill", "white")
                .attr_signal("r", radius.signal().map(ease::linear_unit(10, 100))));

    Task::start(async move {
        loop {
            radius.go_to(1);
            cx.go_to(1);
            cy.go_to(1);

            Timer::sleep(1500).await;
            radius.go_to(0.5);
            cx.go_to(0.7);
            cy.go_to(0.3);

            Timer::sleep(1500).await;
            radius.go_to(0.25);
            cx.go_to(0.5);
            cy.go_to(0.8);

            Timer::sleep(1500).await;
            radius.go_to(0.);
            cx.go_to(0.05);
            cy.go_to(0.5);

            Timer::sleep(1500).await;
        }
    });

    circle
}

fn main() {
    start_app("app", root);
}
