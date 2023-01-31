use log::*;
use std::thread;

use esp_idf_hal::delay::{FreeRtos, BLOCK};
use esp_idf_hal::rmt::{PinState, Pulse, Receive, RxRmtDriver};

use crossbeam_channel::Sender;

const SYNC_PULSE: u16 = 620;
const SYNC_MARGIN: u16 = 60;
const SHORT_PULSE: u16 = 210;
const SHORT_MARGIN: u16 = 100;
const LONG_PULSE: u16 = 420;
const LONG_MARGIN: u16 = 90;

pub enum Acurite5n1Message {
    Type1(MessageType1),
    Type8(MessageType8),
}

pub struct MessageHeader {
    pub channel_number: u8,
    pub report_number: u8,
    pub product_id: u16,
    pub status: u8,
}

pub struct MessageType1 {
    pub header: MessageHeader,
    pub wind_speed: u8,
    pub wind_direction: u8,
    pub rain_bucket_tips: u16,
}

pub struct MessageType8 {
    pub header: MessageHeader,
    pub wind_speed: u8,
    pub temperature: u16,
    pub humidity: u8,
}

pub struct Decoder {
    pub decoded_message: u64,
    sync_count: u8,
    bit_count: u16,
    sync_found: bool,
    found_message: bool,
}

impl Default for Decoder {
    fn default() -> Self {
        Self {
            decoded_message: 0,
            sync_count: 0,
            bit_count: 0,
            sync_found: false,
            found_message: false,
        }
    }
}

pub struct Acurite5n1 {
    tx1: Sender<Acurite5n1Message>,
    rmt_rx: RxRmtDriver<'static>,
    rmt_rx_buf_size: usize,
    decoder: Decoder,
}

impl Acurite5n1 {
    pub fn new(
        tx1: Sender<Acurite5n1Message>,
        rmt_rx: RxRmtDriver<'static>,
        rmt_rx_buf_size: usize,
    ) -> Self {
        Self {
            tx1,
            rmt_rx,
            rmt_rx_buf_size,
            decoder: Decoder::default(),
        }
    }

    pub fn start(mut self) {
        println!("Starting Acurite5n1 Thread");

        let _acurite5n1_thread = thread::spawn(move || {
            let mut pulses = vec![(Pulse::zero(), Pulse::zero()); self.rmt_rx_buf_size];
            let mut message_pulses: Vec<u16> = Vec::new();

            self.rmt_rx.start().unwrap();

            loop {
                // Block until rmt items are available
                let rmt_items = self.rmt_rx.receive(&mut pulses, BLOCK).unwrap();

                match rmt_items {
                    Receive::Read(length) => {
                        self.parse_pulse_stream(&mut pulses, length, &mut message_pulses)
                    }
                    Receive::Overflow(len) => println!("pulses buffer overflowed by {}", len),
                    Receive::Timeout => println!("Receiver timeout"),
                }
            }
        });
    }

    fn parse_pulse_stream(
        &mut self,
        the_pulses: &Vec<(Pulse, Pulse)>,
        length: usize,
        message_pulses: &mut Vec<u16>,
    ) {
        // A packet of 3 messages from the Acurite5n1 is typically 206 pulse-pairs
        if length > 0 {
            let pulses = &the_pulses[..length];

            for (pulse0, pulse1) in pulses {
                // We will use the high pulse to determine if the pulse is a sync pulse or a message pulse
                // A high pulse could be in pulse0 or in pulse1 so we need the check which one has the high pulse.
                let high_pulse = match pulse0.pin_state {
                    PinState::High => pulse0.ticks.ticks(),
                    PinState::Low => pulse1.ticks.ticks(),
                };

                if self.decoder.sync_found {
                    // ============================ COLLECT MESSAGE PULSES ============================

                    // Do not include bad pulses in message_pulses, a bad pulse is a pulse greater than or equal to 510uS.
                    if high_pulse < (LONG_PULSE + LONG_MARGIN) {
                        message_pulses.push(high_pulse);
                        self.decoder.bit_count += 1;
                    } else {
                        // If bit count equals zero then this pulse is a sync pulse, if bit
                        // count is not zero then this pulse is not a valid pulse time for message pulses
                        if self.decoder.bit_count != 0 {
                            // The message is corrupt so remove previous pulses from message_pulses
                            // reset decoder so we will start looking for sync pulses again
                            message_pulses
                                .drain(message_pulses.len() - self.decoder.bit_count as usize..);
                            self.clear_decoder();
                        }
                    }

                    // Check to see if we have collected a message.  We will process message(s) after we exit the for
                    // loop and have turned off the rmt receiver.  Doing it this way we should never get a rmt buffer overrun.
                    if self.decoder.bit_count == 64 {
                        //warn!("Message found");
                        self.decoder.found_message = true;

                        // Reset decoder to look for next message
                        self.clear_decoder();
                    }
                } else {
                    // ============================ LOOK FOR SYNC PULSES ============================

                    // Check if hi_time is sync pulse - pulse greater than 550
                    if high_pulse > (SYNC_PULSE - SYNC_MARGIN) {
                        self.decoder.sync_count += 1;
                    } else {
                        self.decoder.sync_count = 0;
                    }

                    // Search for 4 consecutive sync pulses
                    if self.decoder.sync_count == 4 {
                        self.decoder.sync_found = true;
                        //warn!("==== Sync Found ====== ");
                    }
                }
            }

            // If bit count is not 0 then last message is incomplete so remove those pulses from message_pulses
            if self.decoder.bit_count != 0 {
                message_pulses.drain(message_pulses.len() - self.decoder.bit_count as usize..);
            }

            // For debugging
            if length >= 200 {
                //for (pulse0, pulse1) in pulses {
                //    println!(
                //        "{:?}={}   {:?}={}",
                //        pulse0.pin_state,
                //        pulse0.ticks.ticks(),
                //        pulse1.pin_state,
                //        pulse1.ticks.ticks()
                //    );
                //}

                //for pulse in message_pulses.iter() {
                //println!("{}", pulse);
                //}
                //println!("=================================");
            }

            if self.decoder.found_message {
                warn!("Stopping RMT");
                self.rmt_rx.stop().unwrap();

                if self.found_good_message(message_pulses) {
                    self.process_message()
                }

                message_pulses.clear();
                self.decoder.found_message = false;

                // The acurite5n1 sends a message packet (3 messages) approximately every 18 seconds.
                // To be on the safe side we will delay for 17 seconds before restarting the rmt receiver.
                warn!("Starting 17 second delay");
                FreeRtos::delay_ms(17000);

                info!("################  Starting RMT #################");
                self.rmt_rx.start().unwrap();
            }

            self.clear_decoder();
        }
    }

    fn clear_decoder(&mut self) {
        self.decoder.decoded_message = 0;
        self.decoder.sync_count = 0;
        self.decoder.bit_count = 0;
        self.decoder.sync_found = false;
    }

    fn found_good_message(&mut self, message_pulses: &mut [u16]) -> bool {
        let mut bit_count: u8 = 0;
        let mut good_message_found: bool = false;

        'outer: for pulse in message_pulses.iter() {
            // Use pulse width of 310 which is approximately in the middle bteweem short pulse(210) and long pulse(420)
            if pulse > &(SHORT_PULSE + SHORT_MARGIN) {
                self.decoder.decoded_message = (self.decoder.decoded_message << 1) + 1;
            } else {
                self.decoder.decoded_message = self.decoder.decoded_message << 1;
            }

            bit_count = bit_count + 1;

            if bit_count == 64 {
                // We only need one good message since all the messages in message_pulses have same measurement values
                if self.is_message_integrity_good(self.decoder.decoded_message) {
                    warn!("good message = {:#017x}", self.decoder.decoded_message);
                    good_message_found = true;
                    break 'outer;
                }
                bit_count = 0;
            }
        }

        good_message_found
    }

    fn process_message(&self) {
        let channel_number = self.read_bit_field(self.decoder.decoded_message, 63, 62);
        let pid_msb: u16 = self.read_bit_field(self.decoder.decoded_message, 59, 56) as u16;
        let pid_lsb: u16 = self.read_bit_field(self.decoder.decoded_message, 55, 48) as u16;
        let product_id = (pid_msb << 7) | pid_lsb;
        let report_number = self.read_bit_field(self.decoder.decoded_message, 61, 60);
        let status = self.read_bit_field(self.decoder.decoded_message, 47, 44);
        let message_type = self.read_bit_field(self.decoder.decoded_message, 43, 40);

        let wind_speed_msb = self.read_bit_field(self.decoder.decoded_message, 36, 32);
        let wind_speed_lsb = self.read_bit_field(self.decoder.decoded_message, 30, 28);
        let wind_speed = wind_speed_msb << 3 | wind_speed_lsb;

        let message_header = MessageHeader {
            channel_number,
            product_id,
            report_number,
            status,
        };

        if message_type == 1 {
            // Message Type 1
            let wind_direction = self.read_bit_field(self.decoder.decoded_message, 27, 24);
            let rb_tips_msb: u16 = self.read_bit_field(self.decoder.decoded_message, 21, 16) as u16;
            let rb_tips_lsb: u16 = self.read_bit_field(self.decoder.decoded_message, 14, 8) as u16;
            let rain_bucket_tips = (rb_tips_msb << 7) | rb_tips_lsb;

            let message_type_1 = MessageType1 {
                header: (message_header),
                wind_speed,
                wind_direction,
                rain_bucket_tips,
            };
            self.tx1
                .send(Acurite5n1Message::Type1(message_type_1))
                .unwrap();
        } else {
            // Message Type 8
            let temp_msb: u16 = self.read_bit_field(self.decoder.decoded_message, 27, 24) as u16;
            let temp_lsb: u16 = self.read_bit_field(self.decoder.decoded_message, 22, 16) as u16;
            let temperature = (temp_msb << 7) | temp_lsb;
            let humidity = self.read_bit_field(self.decoder.decoded_message, 14, 8);

            let message_type_8 = MessageType8 {
                header: (message_header),
                wind_speed,
                temperature,
                humidity,
            };
            self.tx1
                .send(Acurite5n1Message::Type8(message_type_8))
                .unwrap();
        }
    }

    // Check if message has a valid CRC, Byte 7 = MSB, Byte 0 is LSB = CRC byte
    // The message 0xC8E678815924A5C9 has a valid crc
    fn is_crc_valid(&self, message: u64) -> bool {
        let sum_of_all_bytes: u64 = (message >> 56) // byte 7
                                  + (message >> 48) // byte 6
                                  + (message >> 40) // byte 5
                                  + (message >> 32) // byte 4
                                  + (message >> 24) // byte 3
                                  + (message >> 16) // byte 2
                                  + (message >> 8); // byte 1

        let crc_test_result = (sum_of_all_bytes & 0xFF) == message & 0xFF;

        crc_test_result
    }

    // Get parity of number x. It returns true if x has odd parity, and returns false if x has even parity
    // x = 7 will return true or odd parity
    fn get_parity(&self, mut n: u8) -> bool {
        let mut parity = false;

        while n != 0 {
            parity = !parity;
            n = n & (n - 1);
        }

        parity
    }

    // Check message for even parity on bytes 1 thru 4.  Returns true if bytes 1 thru 4 have even parity
    // The message 0xC8E678815924A5C9 has valid parity
    fn is_parity_valid(&self, message: u64) -> bool {
        let has_even_parity = !(self.get_parity((message >> 32) as u8)) // byte 4
                            & !(self.get_parity((message >> 24) as u8)) // byte 3
                            & !(self.get_parity((message >> 16) as u8)) // byte 2
                            & !(self.get_parity((message >> 8)as u8)); // byte 1

        has_even_parity
    }

    // Check message if message type is valid message and messsage is valid
    fn is_message_integrity_good(&self, message: u64) -> bool {
        let message_type = self.read_bit_field(message, 43, 40);

        (message_type == 1 || message_type == 8)
            && self.is_crc_valid(message)
            && self.is_parity_valid(message)
    }

    // Read a bit field (note: The maximum bits in bit field is 8)
    fn read_bit_field(&self, message: u64, msb_posn: u8, lsb_posn: u8) -> u8 {
        let mut keep_bits_mask = 0;

        for n in lsb_posn..=msb_posn {
            keep_bits_mask |= ((1) as u64) << n;
        }

        let bit_field = (message & keep_bits_mask) >> lsb_posn;

        bit_field as u8
    }
}
