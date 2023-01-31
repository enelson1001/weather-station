use crossbeam_channel::Sender;
use debouncr::{debounce_4, Debouncer, Edge, Repeat4};
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{AnyInputPin, Input, PinDriver},
};

use crate::model::peripherals::ButtonsPeripherals;

#[derive(Debug, Copy, Clone)]
pub enum BtnId {
    Btn1,
    Btn2,
    Btn3,
}

pub enum BtnState {
    Pressed,
    Released,
    Held,
}

#[allow(dead_code)]
#[derive(PartialEq, PartialOrd)]
pub enum UserBtnState {
    Btn1Pressed,
    Btn1Released,
    Btn1Held,
    Btn2Pressed,
    Btn2Released,
    Btn2Held,
    Btn3Pressed,
    Btn3Released,
    Btn3Held,
}

pub struct UserButton {
    btn: PinDriver<'static, AnyInputPin, Input>,
    btn_state: Debouncer<u8, Repeat4>,
    btn_id: BtnId,
}

impl UserButton {
    pub fn button_status(&mut self) -> (BtnId, BtnState) {
        let button_edge = self.btn_state.update(self.btn.is_low());

        if button_edge == Some(Edge::Rising) {
            (self.btn_id, BtnState::Pressed)
        } else if button_edge == Some(Edge::Falling) {
            (self.btn_id, BtnState::Released)
        } else {
            (self.btn_id, BtnState::Held)
        }
    }
}

pub struct UserButtons {
    buttons: [UserButton; 3],
    cbc_tx: Sender<UserBtnState>,
}

impl UserButtons {
    pub fn new(buttons_peripherals: ButtonsPeripherals, cbc_tx: Sender<UserBtnState>) -> Self {
        let btn1 = UserButton {
            btn: PinDriver::input(buttons_peripherals.left_button).unwrap(),
            btn_state: debounce_4(false),
            btn_id: BtnId::Btn1,
        };
        let btn2 = UserButton {
            btn: PinDriver::input(buttons_peripherals.middle_button).unwrap(),
            btn_state: debounce_4(false),
            btn_id: BtnId::Btn2,
        };

        let btn3 = UserButton {
            btn: PinDriver::input(buttons_peripherals.right_button).unwrap(),
            btn_state: debounce_4(false),
            btn_id: BtnId::Btn3,
        };

        Self {
            buttons: [btn1, btn2, btn3],
            cbc_tx: (cbc_tx),
        }
    }

    pub fn start(mut self) {
        // Start user button debounce thread
        let _user_buttons_debounce_thread = std::thread::spawn(move || loop {
            println!("User buttons debounce thread started");

            loop {
                for btn in 0..self.buttons.len() {
                    match self.buttons[btn].button_status() {
                        (BtnId::Btn1, BtnState::Pressed) => {
                            self.cbc_tx.send(UserBtnState::Btn1Pressed).unwrap()
                        }
                        (BtnId::Btn2, BtnState::Pressed) => {
                            self.cbc_tx.send(UserBtnState::Btn2Pressed).unwrap()
                        }
                        (BtnId::Btn3, BtnState::Pressed) => {
                            self.cbc_tx.send(UserBtnState::Btn3Pressed).unwrap()
                        }

                        (BtnId::Btn1, BtnState::Released) => {
                            self.cbc_tx.send(UserBtnState::Btn1Released).unwrap()
                        }
                        (BtnId::Btn2, BtnState::Released) => {
                            self.cbc_tx.send(UserBtnState::Btn2Released).unwrap()
                        }
                        (BtnId::Btn3, BtnState::Released) => {
                            self.cbc_tx.send(UserBtnState::Btn3Released).unwrap()
                        }

                        // Don't care about held state - not implemented
                        _ => (),
                    }
                }

                FreeRtos::delay_ms(20);
            }
        });
    }
}
