use std::convert::TryInto;
use std::error::Error;

// Frame is 7 bytes long
// It always starts with '104' and ends with '22'
// The second byte indicates the key (or other things for desk->panel)
// The penultimate byte is a checksum: summation of bytes 2 through 5 inclusive, modulo 256
// [START,a,b,c,d,CHECKSUM,END]

pub const DATA_FRAME_SIZE: usize = 7;
pub const DATA_FRAME_START: u8 = 104u8;
pub const DATA_FRAME_END: u8 = 22u8;

const RX_HEIGHT_BYTE: u8 = 0u8;
const TX_UP_BYTE: u8 = 1u8;
const TX_DOWN_BYTE: u8 = 2u8;
const TX_NO_KEY_BYTE: u8 = 3u8;
const TX_ONE_BYTE: u8 = 6u8;
const TX_TWO_BYTE: u8 = 7u8;
const TX_THREE_BYTE: u8 = 8u8;
const TX_ONE_RESET_BYTE: u8 = 10u8;
const TX_TWO_RESET_BYTE: u8 = 11u8;
const TX_THREE_RESET_BYTE: u8 = 12u8;

pub type DataFrame = Vec<u8>;

// TODO: Add messages for resetting 1,2,3 keys
#[derive(Debug, PartialEq)]
pub enum TxMessage {
    Up,
    Down,
    One(f32),
    Two(f32),
    Three(f32),
    // ResetOne,
    // ResetTwo,
    // ResetThree,
    NoKey,
    Unknown(u8, u8, u8, u8, u8),
}

impl TxMessage {
    pub fn as_frame(&self) -> DataFrame {
        match *self {
            TxMessage::Up => build_frame(TX_UP_BYTE, 0u8, 0u8),
            TxMessage::Down => build_frame(TX_DOWN_BYTE, 0u8, 0u8),
            TxMessage::NoKey => build_frame(TX_NO_KEY_BYTE, 0u8, 0u8),
            TxMessage::One(target_height) => {
                let (height_msb, height_lsb) = height_to_bytes(target_height, 0.0);
                build_frame(TX_ONE_BYTE, height_lsb, height_msb)
            }
            TxMessage::Two(target_height) => {
                let (height_msb, height_lsb) = height_to_bytes(target_height, 0.0);
                build_frame(TX_TWO_BYTE, height_lsb, height_msb)
            }
            TxMessage::Three(target_height) => {
                let (height_msb, height_lsb) = height_to_bytes(target_height, 0.0);
                build_frame(TX_THREE_BYTE, height_lsb, height_msb)
            }
            TxMessage::Unknown(a, b, c, d, e) => {
                vec![DATA_FRAME_START, a, b, c, d, e, DATA_FRAME_END]
            }
        }
    }

    // TODO: Add messages for resetting 1,2,3 keys
    pub fn from_frame(buf: &DataFrame) -> TxMessage {
        // TODO: validate checksum somewhere. Or don't; just pass it on to desk?
        match buf[2] {
            TX_UP_BYTE => TxMessage::Up,
            TX_DOWN_BYTE => TxMessage::Down,
            TX_NO_KEY_BYTE => TxMessage::NoKey,
            TX_ONE_BYTE => TxMessage::One(bytes_to_height_cm(buf[4], buf[3], 0.0)),
            TX_TWO_BYTE => TxMessage::Two(bytes_to_height_cm(buf[4], buf[3], 0.0)),
            TX_THREE_BYTE => TxMessage::Three(bytes_to_height_cm(buf[4], buf[3], 0.0)),
            // TX_ONE_RESET_BYTE=>
            // TX_TWO_RESET_BYTE=>
            // TX_THREE_RESET_BYTE=>
            _ => TxMessage::Unknown(buf[1], buf[2], buf[3], buf[4], buf[5]),
        }
    }
}

fn build_frame(b2: u8, b3: u8, b4: u8) -> DataFrame {
    vec![
        DATA_FRAME_START,
        1u8,
        b2,
        b3,
        b4,
        checksum(&[1u8, b2, b3, b4]),
        DATA_FRAME_END,
    ]
}

#[derive(Debug, PartialEq)]
pub enum RxMessage {
    Height(f32),
    Unknown(u8, u8, u8, u8, u8),
}

impl RxMessage {
    pub fn as_frame(&self) -> DataFrame {
        match *self {
            RxMessage::Height(h) => {
                // TODO: handle height outside of range
                let (height_msb, height_lsb) = height_to_bytes(h, 65.0);
                build_frame(RX_HEIGHT_BYTE, height_msb, height_lsb)
            }
            RxMessage::Unknown(a, b, c, d, e) => {
                vec![DATA_FRAME_START, a, b, c, d, e, DATA_FRAME_END]
            }
        }
    }

    pub fn from_frame(frame: &DataFrame) -> RxMessage {
        // TODO: validate checksum somewhere. Or don't; just pass it on to panel?
        match frame[2] {
            RX_HEIGHT_BYTE => RxMessage::Height(bytes_to_height_cm(frame[3], frame[4], 65.0)),
            _ => RxMessage::Unknown(frame[1], frame[2], frame[3], frame[4], frame[5]),
        }
    }
}

fn bytes_to_height_cm(msb: u8, lsb: u8, offset_cm: f32) -> f32 {
    (256.0 * msb as f32 + lsb as f32) / 10.0 + offset_cm
}

fn height_to_bytes(height_cm: f32, offset_cm: f32) -> (u8, u8) {
    let net_height_mm = (height_cm - offset_cm) * 10.0;
    let msb = (net_height_mm / 256.0) as u8;

    let lsb = (net_height_mm - (msb as f32 * 256.0)) as u8;
    (msb, lsb)
}

fn checksum(b: &[u8]) -> u8 {
    (b.iter().map(|x| *x as usize).sum::<usize>() % 256) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_message_from_frame() {
        assert_eq!(
            TxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                6u8,
                0u8,
                0u8,
                7u8,
                DATA_FRAME_END
            ]),
            TxMessage::One(0.0),
        );

        assert_eq!(
            TxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                7u8,
                0u8,
                0u8,
                8u8,
                DATA_FRAME_END
            ]),
            TxMessage::Two(0.0),
        );

        assert_eq!(
            TxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                8u8,
                0u8,
                0u8,
                9u8,
                DATA_FRAME_END
            ]),
            TxMessage::Three(0.0),
        );

        assert_eq!(
            TxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                6u8,
                138u8,
                2u8,
                147u8,
                DATA_FRAME_END
            ]),
            TxMessage::One(65.0),
        );

        assert_eq!(
            TxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                7u8,
                138u8,
                2u8,
                148u8,
                DATA_FRAME_END
            ]),
            TxMessage::Two(65.0),
        );

        assert_eq!(
            TxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                8u8,
                138u8,
                2u8,
                149u8,
                DATA_FRAME_END
            ]),
            TxMessage::Three(65.0),
        );

        assert_eq!(
            TxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                6u8,
                143u8,
                2u8,
                152u8,
                DATA_FRAME_END
            ]),
            TxMessage::One(65.5),
        );

        assert_eq!(
            TxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                6u8,
                232u8,
                3u8,
                242u8,
                DATA_FRAME_END
            ]),
            TxMessage::One(100.0),
        );

        assert_eq!(
            TxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                6u8,
                253u8,
                2u8,
                6u8,
                DATA_FRAME_END
            ]),
            TxMessage::One(76.5),
        );

        assert_eq!(
            TxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                6u8,
                2u8,
                3u8,
                12u8,
                DATA_FRAME_END
            ]),
            TxMessage::One(77.0),
        );

        assert_eq!(
            TxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                6u8,
                252u8,
                3u8,
                6u8,
                DATA_FRAME_END
            ]),
            TxMessage::One(102.0),
        );

        assert_eq!(
            TxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                6u8,
                1u8,
                4u8,
                12u8,
                DATA_FRAME_END
            ]),
            TxMessage::One(102.5),
        );

        assert_eq!(
            TxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                6u8,
                15u8,
                5u8,
                27u8,
                DATA_FRAME_END
            ]),
            TxMessage::One(129.5),
        );
    }

    #[test]
    fn test_tx_message_as_frame() {
        assert_eq!(
            TxMessage::Up.as_frame(),
            vec![
                DATA_FRAME_START,
                1u8,
                TX_UP_BYTE,
                0u8,
                0u8,
                2u8,
                DATA_FRAME_END
            ],
        );

        assert_eq!(
            TxMessage::Down.as_frame(),
            vec![
                DATA_FRAME_START,
                1u8,
                TX_DOWN_BYTE,
                0u8,
                0u8,
                3u8,
                DATA_FRAME_END
            ],
        );

        assert_eq!(
            TxMessage::NoKey.as_frame(),
            vec![
                DATA_FRAME_START,
                1u8,
                TX_NO_KEY_BYTE,
                0u8,
                0u8,
                4u8,
                DATA_FRAME_END
            ],
        );

        assert_eq!(
            TxMessage::Unknown(99u8, 64u8, 254u8, 1u8, 98u8).as_frame(),
            vec![
                DATA_FRAME_START,
                99u8,
                64u8,
                254u8,
                1u8,
                98u8,
                DATA_FRAME_END
            ],
        );

        assert_eq!(
            TxMessage::One(0.0).as_frame(),
            vec![DATA_FRAME_START, 1u8, 6u8, 0u8, 0u8, 7u8, DATA_FRAME_END]
        );

        assert_eq!(
            TxMessage::Two(0.0).as_frame(),
            vec![DATA_FRAME_START, 1u8, 7u8, 0u8, 0u8, 8u8, DATA_FRAME_END]
        );

        assert_eq!(
            TxMessage::Three(0.0).as_frame(),
            vec![DATA_FRAME_START, 1u8, 8u8, 0u8, 0u8, 9u8, DATA_FRAME_END]
        );

        assert_eq!(
            TxMessage::One(65.0).as_frame(),
            vec![
                DATA_FRAME_START,
                1u8,
                6u8,
                138u8,
                2u8,
                147u8,
                DATA_FRAME_END
            ]
        );

        assert_eq!(
            TxMessage::Two(65.0).as_frame(),
            vec![
                DATA_FRAME_START,
                1u8,
                7u8,
                138u8,
                2u8,
                148u8,
                DATA_FRAME_END
            ]
        );

        assert_eq!(
            TxMessage::Three(65.0).as_frame(),
            vec![
                DATA_FRAME_START,
                1u8,
                8u8,
                138u8,
                2u8,
                149u8,
                DATA_FRAME_END
            ]
        );

        assert_eq!(
            TxMessage::One(65.5).as_frame(),
            vec![
                DATA_FRAME_START,
                1u8,
                6u8,
                143u8,
                2u8,
                152u8,
                DATA_FRAME_END
            ]
        );

        assert_eq!(
            TxMessage::One(100.0).as_frame(),
            vec![
                DATA_FRAME_START,
                1u8,
                6u8,
                232u8,
                3u8,
                242u8,
                DATA_FRAME_END
            ]
        );

        assert_eq!(
            TxMessage::One(76.5).as_frame(),
            vec![DATA_FRAME_START, 1u8, 6u8, 253u8, 2u8, 6u8, DATA_FRAME_END]
        );

        assert_eq!(
            TxMessage::One(77.0).as_frame(),
            vec![DATA_FRAME_START, 1u8, 6u8, 2u8, 3u8, 12u8, DATA_FRAME_END]
        );

        assert_eq!(
            TxMessage::One(102.0).as_frame(),
            vec![DATA_FRAME_START, 1u8, 6u8, 252u8, 3u8, 6u8, DATA_FRAME_END]
        );

        assert_eq!(
            TxMessage::One(102.5).as_frame(),
            vec![DATA_FRAME_START, 1u8, 6u8, 1u8, 4u8, 12u8, DATA_FRAME_END]
        );

        assert_eq!(
            TxMessage::One(129.5).as_frame(),
            vec![DATA_FRAME_START, 1u8, 6u8, 15u8, 5u8, 27u8, DATA_FRAME_END]
        );
    }

    #[test]
    fn test_rx_message_as_frame() {
        // TODO: test < 65.0
        // TODO: test > 129.5
        // TODO: test intervals of something other than 5mm / 0.5cm

        assert_eq!(
            RxMessage::Height(65.0).as_frame(),
            vec![DATA_FRAME_START, 1u8, 0u8, 0u8, 0u8, 1u8, DATA_FRAME_END],
        );

        assert_eq!(
            RxMessage::Height(65.5).as_frame(),
            vec![DATA_FRAME_START, 1u8, 0u8, 0u8, 5u8, 6u8, DATA_FRAME_END],
        );

        assert_eq!(
            RxMessage::Height(100.0).as_frame(),
            vec![DATA_FRAME_START, 1u8, 0u8, 1u8, 94u8, 96u8, DATA_FRAME_END],
        );

        assert_eq!(
            RxMessage::Height(90.5).as_frame(),
            vec![DATA_FRAME_START, 1u8, 0u8, 0u8, 255u8, 0u8, DATA_FRAME_END],
        );

        assert_eq!(
            RxMessage::Height(91.0).as_frame(),
            vec![DATA_FRAME_START, 1u8, 0u8, 1u8, 4u8, 6u8, DATA_FRAME_END],
        );

        assert_eq!(
            RxMessage::Height(116.0).as_frame(),
            vec![DATA_FRAME_START, 1u8, 0u8, 1u8, 254u8, 0u8, DATA_FRAME_END],
        );

        assert_eq!(
            RxMessage::Height(116.5).as_frame(),
            vec![DATA_FRAME_START, 1u8, 0u8, 2u8, 3u8, 6u8, DATA_FRAME_END],
        );

        assert_eq!(
            RxMessage::Height(129.5).as_frame(),
            vec![
                DATA_FRAME_START,
                1u8,
                0u8,
                2u8,
                133u8,
                136u8,
                DATA_FRAME_END
            ],
        );
    }

    #[test]
    fn test_rx_message_from_frame() {
        assert_eq!(
            RxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                0u8,
                0u8,
                0u8,
                1u8,
                DATA_FRAME_END
            ]),
            RxMessage::Height(65.0),
        );

        assert_eq!(
            RxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                0u8,
                0u8,
                5u8,
                6u8,
                DATA_FRAME_END
            ]),
            RxMessage::Height(65.5),
        );

        assert_eq!(
            RxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                0u8,
                1u8,
                94u8,
                96u8,
                DATA_FRAME_END
            ]),
            RxMessage::Height(100.0),
        );

        assert_eq!(
            RxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                0u8,
                0u8,
                255u8,
                0u8,
                DATA_FRAME_END
            ]),
            RxMessage::Height(90.5),
        );

        assert_eq!(
            RxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                0u8,
                1u8,
                4u8,
                6u8,
                DATA_FRAME_END
            ]),
            RxMessage::Height(91.0),
        );

        assert_eq!(
            RxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                0u8,
                1u8,
                254u8,
                0u8,
                DATA_FRAME_END
            ]),
            RxMessage::Height(116.0),
        );

        assert_eq!(
            RxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                0u8,
                2u8,
                3u8,
                6u8,
                DATA_FRAME_END
            ]),
            RxMessage::Height(116.5),
        );

        assert_eq!(
            RxMessage::from_frame(&vec![
                DATA_FRAME_START,
                1u8,
                0u8,
                2u8,
                133u8,
                136u8,
                DATA_FRAME_END
            ]),
            RxMessage::Height(129.5),
        );
    }
}
