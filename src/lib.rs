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

const TX_UP_BYTE: u8 = 1u8;
const TX_DOWN_BYTE: u8 = 2u8;
const TX_NO_KEY_BYTE: u8 = 3u8;
const TX_ONE_BYTE: u8 = 6u8;
const TX_TWO_BYTE: u8 = 7u8;
const TX_THREE_BYTE: u8 = 8u8;
const TX_ONE_RESET_BYTE: u8 = 10u8;
const TX_TWO_RESET_BYTE: u8 = 11u8;
const TX_THREE_RESET_BYTE: u8 = 12u8;

const TX_UP_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 1u8, 0u8, 0u8, 2u8, DATA_FRAME_END];
const TX_DOWN_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 2u8, 0u8, 0u8, 3u8, DATA_FRAME_END];
const TX_NOKEY_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 3u8, 0u8, 0u8, 4u8, DATA_FRAME_END];

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
            TxMessage::Up => TX_UP_FRAME.to_vec(),
            TxMessage::Down => TX_DOWN_FRAME.to_vec(),
            TxMessage::One(target_height) => foo(TX_ONE_BYTE, target_height),
            TxMessage::Two(target_height) => foo(TX_TWO_BYTE, target_height),
            TxMessage::Three(target_height) => foo(TX_THREE_BYTE, target_height),
            TxMessage::NoKey => TX_NOKEY_FRAME.to_vec(),
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
        // match buf.as_slice().try_into().unwrap() {
        //     TX_UP_FRAME => TxMessage::Up,
        //     TX_DOWN_FRAME => TxMessage::Down,
        //     TX_ONE_FRAME => TxMessage::One(0.0),
        //     TX_TWO_FRAME => TxMessage::Two(0.0),
        //     TX_THREE_FRAME => TxMessage::Three(0.0),
        //     TX_NOKEY_FRAME => TxMessage::NoKey,
        //     _ => TxMessage::Unknown(buf[1], buf[2], buf[3], buf[4], buf[5]),
        // }
    }
}

fn foo(b2: u8, target_height: f32) -> DataFrame {
    // TODO: handle 0 target height (i.e. unset)
    // TODO: handle height outside of range

    let (height_msb, height_lsb) = height_to_bytes(target_height, 0.0);

    let mut frame = vec![0u8; DATA_FRAME_SIZE];
    frame[0] = DATA_FRAME_START;
    frame[1] = 1u8;
    frame[2] = b2;
    frame[3] = height_lsb;
    frame[4] = height_msb;
    frame[5] = checksum(&frame[1..5]);
    frame[6] = DATA_FRAME_END;

    frame
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

                let mut frame = vec![0u8; DATA_FRAME_SIZE];
                frame[0] = DATA_FRAME_START;
                frame[1] = 1u8;
                frame[2] = 0u8;
                frame[3] = height_msb;
                frame[4] = height_lsb;
                frame[5] = checksum(&frame[1..5]);
                frame[6] = DATA_FRAME_END;

                frame
            }
            RxMessage::Unknown(a, b, c, d, e) => {
                vec![DATA_FRAME_START, a, b, c, d, e, DATA_FRAME_END]
            }
        }
    }

    pub fn from_frame(frame: &DataFrame) -> RxMessage {
        // TODO: validate checksum somewhere. Or don't; just pass it on to panel?
        let buf: [u8; DATA_FRAME_SIZE] = frame.as_slice().try_into().unwrap();
        // println!("from_frame: {:?}, buf[1..2] = {:?}", buf, &buf[1..3]);
        match buf[1..3] {
            [1u8, 0u8] => RxMessage::Height(bytes_to_height_cm(buf[3], buf[4], 65.0)),
            _ => RxMessage::Unknown(buf[1], buf[2], buf[3], buf[4], buf[5]),
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
