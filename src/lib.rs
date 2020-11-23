use std::convert::TryInto;
use std::error::Error;

pub const DATA_FRAME_SIZE: usize = 7;
pub const DATA_FRAME_START: u8 = 104u8;
pub const DATA_FRAME_END: u8 = 22u8;

const TX_UP_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 1u8, 0u8, 0u8, 2u8, DATA_FRAME_END];
const TX_DOWN_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 2u8, 0u8, 0u8, 3u8, DATA_FRAME_END];
const TX_ONE_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 6u8, 0u8, 0u8, 7u8, DATA_FRAME_END];
const TX_TWO_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 7u8, 0u8, 0u8, 8u8, DATA_FRAME_END];
const TX_THREE_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 8u8, 0u8, 0u8, 9u8, DATA_FRAME_END];
const TX_NOKEY_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 3u8, 0u8, 0u8, 4u8, DATA_FRAME_END];
const TX_UNKNOWN_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 1u8, 1u8, 1u8, 1u8, DATA_FRAME_END];
// TODO: make frame part of Unknown enum

pub type DataFrame = Vec<u8>;

#[derive(Debug, PartialEq)]
pub enum TxMessage {
    Up,
    Down,
    One,
    Two,
    Three,
    NoKey,
    Unknown,
}

impl TxMessage {
    pub fn as_frame(&self) -> DataFrame {
        match *self {
            TxMessage::Up => TX_UP_FRAME.to_vec(),
            TxMessage::Down => TX_DOWN_FRAME.to_vec(),
            TxMessage::One => TX_ONE_FRAME.to_vec(),
            TxMessage::Two => TX_TWO_FRAME.to_vec(),
            TxMessage::Three => TX_THREE_FRAME.to_vec(),
            TxMessage::NoKey => TX_NOKEY_FRAME.to_vec(),
            TxMessage::Unknown => TX_UNKNOWN_FRAME.to_vec(),
        }
    }

    pub fn from_frame(buf: &DataFrame) -> TxMessage {
        match buf.as_slice().try_into().unwrap() {
            TX_UP_FRAME => TxMessage::Up,
            TX_DOWN_FRAME => TxMessage::Down,
            TX_ONE_FRAME => TxMessage::One,
            TX_TWO_FRAME => TxMessage::Two,
            TX_THREE_FRAME => TxMessage::Three,
            TX_NOKEY_FRAME => TxMessage::NoKey,
            _ => TxMessage::Unknown,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RxMessage {
    Height(f32),
    Unknown,
}

impl RxMessage {
    pub fn as_frame(&self) -> DataFrame {
        match *self {
            RxMessage::Height(h) => create_height_frame_from_height_cm(h).unwrap().to_vec(),
            RxMessage::Unknown => TX_UNKNOWN_FRAME.to_vec(),
        }
    }

    pub fn from_frame(frame: &DataFrame) -> RxMessage {
        let buf: [u8; DATA_FRAME_SIZE] = frame.as_slice().try_into().unwrap();
        // println!("from_frame: {:?}, buf[1..2] = {:?}", buf, &buf[1..3]);
        match buf[1..3] {
            [1u8, 0u8] => RxMessage::Height(read_height_cm_from_frame(frame).unwrap()),
            _ => RxMessage::Unknown,
        }
    }
}

pub fn read_height_cm_from_frame(frame: &DataFrame) -> Result<f32, Box<dyn Error>> {
    Ok((256 * frame[3] as isize + frame[4] as isize + 650) as f32 / 10.0)
}

pub fn create_height_frame_from_height_cm(height: f32) -> Result<DataFrame, Box<dyn Error>> {
    let height_in_mm = height * 10.0;
    let x = height_in_mm - 650.0;
    let x2 = (x / 256.0) as u8;

    let mut frame = vec![0u8; DATA_FRAME_SIZE];
    frame[0] = DATA_FRAME_START;
    frame[DATA_FRAME_SIZE - 1] = DATA_FRAME_END;
    frame[1] = 1u8;
    frame[2] = 0u8;
    frame[3] = x2;
    frame[4] = (x - (x2 as f32 * 256.0)) as u8;
    frame[5] = ((frame[3] as usize + frame[4] as usize + 1) % 256) as u8;

    Ok(frame)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_height_frame_from_height_cm() {
        // TODO: test < 65.0
        // TODO: test > 129.5
        // TODO: test intervals of something other than 5mm / 0.5cm

        assert_eq!(
            create_height_frame_from_height_cm(65.0).unwrap(),
            [DATA_FRAME_START, 1u8, 0u8, 0u8, 0u8, 1u8, DATA_FRAME_END].to_vec()
        );

        assert_eq!(
            create_height_frame_from_height_cm(65.5).unwrap(),
            [DATA_FRAME_START, 1u8, 0u8, 0u8, 5u8, 6u8, DATA_FRAME_END].to_vec()
        );

        assert_eq!(
            create_height_frame_from_height_cm(90.5).unwrap(),
            [DATA_FRAME_START, 1u8, 0u8, 0u8, 255u8, 0u8, DATA_FRAME_END].to_vec()
        );

        assert_eq!(
            create_height_frame_from_height_cm(129.5).unwrap(),
            [
                DATA_FRAME_START,
                1u8,
                0u8,
                2u8,
                133u8,
                136u8,
                DATA_FRAME_END
            ]
            .to_vec()
        );
    }
}
