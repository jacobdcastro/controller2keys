use enigo::{Enigo, KeyboardControllable, MouseControllable};
use gilrs::{Button, Event, EventType, Gilrs};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::{thread, time::Duration};

#[cfg(target_os = "macos")]
use libc;

// define our button to key mappings
lazy_static! {
    static ref BUTTON_MAP: HashMap<Button, enigo::Key> = {
        let mut m = HashMap::new();
        // face buttons
        m.insert(Button::South, enigo::Key::Space);          // a button -> spacebar
        m.insert(Button::East, enigo::Key::Shift);           // b button -> shift
        m.insert(Button::West, enigo::Key::Layout('e'));     // x button -> 'e' key
        m.insert(Button::North, enigo::Key::Layout('e'));    // y button -> 'e' key

        // d-pad
        m.insert(Button::DPadUp, enigo::Key::F5);           // up -> F5
        m.insert(Button::DPadDown, enigo::Key::Layout('q')); // down -> 'q'
        m.insert(Button::DPadLeft, enigo::Key::Layout('b')); // left -> 'b'
        m.insert(Button::DPadRight, enigo::Key::Layout('/')); // right -> '/'

        // stick buttons
        m.insert(Button::LeftThumb, enigo::Key::Control);    // left stick press -> ctrl
        m.insert(Button::RightThumb, enigo::Key::Layout('v')); // keeping this as is

        // menu buttons (keeping these as is)
        m.insert(Button::Select, enigo::Key::Tab);           // select/back -> tab
        m.insert(Button::Start, enigo::Key::Escape);         // start/menu -> escape
        m
    };

    // add new map for mouse buttons
    static ref MOUSE_BUTTON_MAP: HashMap<Button, enigo::MouseButton> = {
        let mut m = HashMap::new();
        m.insert(Button::RightTrigger2, enigo::MouseButton::Left);
        m.insert(Button::LeftTrigger2, enigo::MouseButton::Right);
        m
    };
}

fn main() {
    // set high priority for this process
    #[cfg(target_os = "linux")]
    unsafe {
        // Set Linux real-time priority
        let pid = libc::getpid();
        let mut param: libc::sched_param = std::mem::zeroed();
        param.sched_priority = libc::sched_get_priority_max(libc::SCHED_RR);
        libc::sched_setscheduler(pid, libc::SCHED_RR, &param);
    }

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Threading::{
            GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_TIME_CRITICAL,
        };
        unsafe {
            SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_TIME_CRITICAL);
        }
    }

    #[cfg(target_os = "macos")]
    unsafe {
        let thread_id = libc::pthread_self();
        let policy = libc::SCHED_RR;
        let mut param: libc::sched_param = std::mem::zeroed();
        param.sched_priority = libc::sched_get_priority_max(policy);
        libc::pthread_setschedparam(thread_id, policy, &param);
    }

    // initialize gamepad system
    let mut gilrs = Gilrs::new().expect("failed to initialize gilrs");

    // initialize keyboard simulator
    let mut enigo = Enigo::new();

    // reduce sleep time even further for more frequent polling
    let poll_rate = Duration::from_millis(4); // ~250Hz polling

    println!("controller2keys started - waiting for controller input...");

    // track active gamepad
    let mut active_gamepad = None;

    loop {
        // update active gamepad
        if active_gamepad.is_none() {
            active_gamepad = gilrs.gamepads().next().map(|(id, _)| id);
        }

        // handle events
        while let Some(Event { id, event, time: _ }) = gilrs.next_event() {
            match event {
                EventType::ButtonPressed(button, _) => {
                    match button {
                        Button::LeftTrigger => {
                            // scroll left/down (negative)
                            enigo.mouse_scroll_y(-1);
                            println!("left shoulder pressed -> simulating scroll down");
                        }
                        Button::RightTrigger => {
                            // scroll right/up (positive)
                            enigo.mouse_scroll_y(1);
                            println!("right shoulder pressed -> simulating scroll up");
                        }
                        _ => {
                            if let Some(&key) = BUTTON_MAP.get(&button) {
                                enigo.key_down(key);
                                println!(
                                    "button {:?} pressed -> simulating key down {:?}",
                                    button, key
                                );
                            } else if let Some(&mouse_button) = MOUSE_BUTTON_MAP.get(&button) {
                                enigo.mouse_down(mouse_button);
                                println!(
                                    "button {:?} pressed -> simulating mouse down {:?}",
                                    button, mouse_button
                                );
                            }
                        }
                    }
                }
                EventType::ButtonReleased(button, _) => {
                    if let Some(&key) = BUTTON_MAP.get(&button) {
                        enigo.key_up(key);
                        println!(
                            "button {:?} released -> simulating key up {:?}",
                            button, key
                        );
                    } else if let Some(&mouse_button) = MOUSE_BUTTON_MAP.get(&button) {
                        enigo.mouse_up(mouse_button);
                        println!(
                            "button {:?} released -> simulating mouse up {:?}",
                            button, mouse_button
                        );
                    }
                }
                EventType::AxisChanged(axis, value, _) => {
                    // handle analog inputs with a smaller deadzone for better responsiveness
                    let deadzone = 0.15; // reduced deadzone for more sensitive input
                    match axis {
                        gilrs::Axis::LeftStickX => {
                            if value.abs() > deadzone {
                                if value > 0.0 {
                                    enigo.key_down(enigo::Key::Layout('d'));
                                    enigo.key_up(enigo::Key::Layout('a')); // ensure opposite key is released
                                } else {
                                    enigo.key_down(enigo::Key::Layout('a'));
                                    enigo.key_up(enigo::Key::Layout('d')); // ensure opposite key is released
                                }
                            } else {
                                // in deadzone - release both keys
                                enigo.key_up(enigo::Key::Layout('d'));
                                enigo.key_up(enigo::Key::Layout('a'));
                            }
                        }
                        gilrs::Axis::LeftStickY => {
                            if value.abs() > deadzone {
                                if value > 0.0 {
                                    enigo.key_down(enigo::Key::Layout('w'));
                                    enigo.key_up(enigo::Key::Layout('s')); // ensure opposite key is released
                                } else {
                                    enigo.key_down(enigo::Key::Layout('s'));
                                    enigo.key_up(enigo::Key::Layout('w')); // ensure opposite key is released
                                }
                            } else {
                                // in deadzone - release both keys
                                enigo.key_up(enigo::Key::Layout('w'));
                                enigo.key_up(enigo::Key::Layout('s'));
                            }
                        }
                        gilrs::Axis::RightStickX => {
                            if value.abs() > deadzone {
                                // increase sensitivity and use linear response for more direct control
                                let mouse_speed = 50.0; // significantly increased sensitivity
                                let movement = (value * mouse_speed) as i32;
                                enigo.mouse_move_relative(movement, 0);
                                println!("Right X: {} -> Mouse X: {}", value, movement);
                                // debug output
                            }
                        }
                        gilrs::Axis::RightStickY => {
                            if value.abs() > deadzone {
                                let mouse_speed = 50.0; // significantly increased sensitivity
                                let movement = (-value * mouse_speed) as i32;
                                enigo.mouse_move_relative(0, movement);
                                println!("Right Y: {} -> Mouse Y: {}", value, movement);
                                // debug output
                            }
                        }
                        _ => (),
                    }
                }
                _ => (), // ignore other events
            }
        }

        // prevent CPU from maxing out but keep responsive
        thread::sleep(poll_rate);
    }
}
