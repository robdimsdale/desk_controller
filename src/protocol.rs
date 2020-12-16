// Frame is 7 bytes long
// It always starts with '104' and ends with '22'
// The second byte indicates the key (or other things for desk->panel)
// The penultimate byte is a checksum: summation of bytes 2 through 5 inclusive, modulo 256
// [START,a,b,c,d,CHECKSUM,END]

pub const DATA_FRAME_SIZE: usize = 7;
pub const DATA_FRAME_START_BYTE: u8 = 104u8;
pub const DATA_FRAME_END_BYTE: u8 = 22u8;

const DESK_TO_PANEL_HEIGHT_BYTE: u8 = 0u8;

const PANEL_TO_DESK_UP_BYTE: u8 = 1u8;
const PANEL_TO_DESK_DOWN_BYTE: u8 = 2u8;
const PANEL_TO_DESK_NO_KEY_BYTE: u8 = 3u8;
const PANEL_TO_DESK_DESK_RESET_BYTE: u8 = 4u8;
const PANEL_TO_DESK_ONE_BYTE: u8 = 6u8;
const PANEL_TO_DESK_TWO_BYTE: u8 = 7u8;
const PANEL_TO_DESK_THREE_BYTE: u8 = 8u8;
const PANEL_TO_DESK_RESET_ONE_BYTE: u8 = 10u8;
const PANEL_TO_DESK_RESET_TWO_BYTE: u8 = 11u8;
const PANEL_TO_DESK_RESET_THREE_BYTE: u8 = 12u8;

pub type DataFrame = Vec<u8>;

#[derive(Debug, PartialEq)]
pub enum PanelToDeskMessage {
    Up,
    Down,
    NoKey,
    DeskReset,
    One(f32),
    Two(f32),
    Three(f32),
    ResetOne,
    ResetTwo,
    ResetThree,
    Unknown(u8, u8, u8, u8, u8),
}

impl PanelToDeskMessage {
    pub fn as_frame(&self) -> DataFrame {
        match *self {
            PanelToDeskMessage::Up => build_frame(PANEL_TO_DESK_UP_BYTE, 0u8, 0u8),
            PanelToDeskMessage::Down => build_frame(PANEL_TO_DESK_DOWN_BYTE, 0u8, 0u8),
            PanelToDeskMessage::NoKey => build_frame(PANEL_TO_DESK_NO_KEY_BYTE, 0u8, 0u8),
            PanelToDeskMessage::DeskReset => build_frame(PANEL_TO_DESK_DESK_RESET_BYTE, 0u8, 0u8),
            PanelToDeskMessage::One(target_height) => {
                let (height_msb, height_lsb) = height_to_bytes(target_height, 0.0);
                build_frame(PANEL_TO_DESK_ONE_BYTE, height_lsb, height_msb)
            }
            PanelToDeskMessage::Two(target_height) => {
                let (height_msb, height_lsb) = height_to_bytes(target_height, 0.0);
                build_frame(PANEL_TO_DESK_TWO_BYTE, height_lsb, height_msb)
            }
            PanelToDeskMessage::Three(target_height) => {
                let (height_msb, height_lsb) = height_to_bytes(target_height, 0.0);
                build_frame(PANEL_TO_DESK_THREE_BYTE, height_lsb, height_msb)
            }
            PanelToDeskMessage::ResetOne => build_frame(PANEL_TO_DESK_RESET_ONE_BYTE, 0u8, 0u8),
            PanelToDeskMessage::ResetTwo => build_frame(PANEL_TO_DESK_RESET_TWO_BYTE, 0u8, 0u8),
            PanelToDeskMessage::ResetThree => build_frame(PANEL_TO_DESK_RESET_THREE_BYTE, 0u8, 0u8),
            PanelToDeskMessage::Unknown(a, b, c, d, e) => {
                vec![DATA_FRAME_START_BYTE, a, b, c, d, e, DATA_FRAME_END_BYTE]
            }
        }
    }

    pub fn from_frame(buf: &DataFrame) -> PanelToDeskMessage {
        // TODO: validate checksum somewhere. Or don't; just pass it on to desk?
        match buf[2] {
            PANEL_TO_DESK_UP_BYTE => PanelToDeskMessage::Up,
            PANEL_TO_DESK_DOWN_BYTE => PanelToDeskMessage::Down,
            PANEL_TO_DESK_NO_KEY_BYTE => PanelToDeskMessage::NoKey,
            PANEL_TO_DESK_DESK_RESET_BYTE => PanelToDeskMessage::DeskReset,
            PANEL_TO_DESK_ONE_BYTE => {
                PanelToDeskMessage::One(bytes_to_height_cm(buf[4], buf[3], 0.0))
            }
            PANEL_TO_DESK_TWO_BYTE => {
                PanelToDeskMessage::Two(bytes_to_height_cm(buf[4], buf[3], 0.0))
            }
            PANEL_TO_DESK_THREE_BYTE => {
                PanelToDeskMessage::Three(bytes_to_height_cm(buf[4], buf[3], 0.0))
            }
            PANEL_TO_DESK_RESET_ONE_BYTE => PanelToDeskMessage::ResetOne,
            PANEL_TO_DESK_RESET_TWO_BYTE => PanelToDeskMessage::ResetTwo,
            PANEL_TO_DESK_RESET_THREE_BYTE => PanelToDeskMessage::ResetThree,
            _ => PanelToDeskMessage::Unknown(buf[1], buf[2], buf[3], buf[4], buf[5]),
        }
    }
}

pub fn validate_frame(frame: &DataFrame) -> bool {
    if frame.len() != DATA_FRAME_SIZE {
        return false;
    }

    if frame[0] != DATA_FRAME_START_BYTE {
        return false;
    }

    if frame[DATA_FRAME_SIZE - 1] != DATA_FRAME_END_BYTE {
        return false;
    }

    return true;
}

fn build_frame(b2: u8, b3: u8, b4: u8) -> DataFrame {
    vec![
        DATA_FRAME_START_BYTE,
        1u8,
        b2,
        b3,
        b4,
        checksum(&[1u8, b2, b3, b4]),
        DATA_FRAME_END_BYTE,
    ]
}

#[derive(Debug, PartialEq)]
pub enum DeskToPanelMessage {
    Height(f32),
    Unknown(u8, u8, u8, u8, u8),
}

impl DeskToPanelMessage {
    pub fn as_frame(&self) -> DataFrame {
        match *self {
            DeskToPanelMessage::Height(h) => {
                // TODO: handle height outside of range
                let (height_msb, height_lsb) = height_to_bytes(h, 65.0);
                build_frame(DESK_TO_PANEL_HEIGHT_BYTE, height_msb, height_lsb)
            }
            DeskToPanelMessage::Unknown(a, b, c, d, e) => {
                vec![DATA_FRAME_START_BYTE, a, b, c, d, e, DATA_FRAME_END_BYTE]
            }
        }
    }

    pub fn from_frame(frame: &DataFrame) -> DeskToPanelMessage {
        // TODO: validate checksum somewhere. Or don't; just pass it on to panel?
        match frame[2] {
            DESK_TO_PANEL_HEIGHT_BYTE => {
                DeskToPanelMessage::Height(bytes_to_height_cm(frame[3], frame[4], 65.0))
            }
            _ => DeskToPanelMessage::Unknown(frame[1], frame[2], frame[3], frame[4], frame[5]),
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
    // TODO: can we do the modulo inline to avoid up-casting to usize? Is it worth it?
    (b.iter().map(|x| *x as usize).sum::<usize>() % 256) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_to_desk_message_from_frame() {
        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_UP_BYTE,
                0u8,
                0u8,
                2u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::Up,
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_DOWN_BYTE,
                0u8,
                0u8,
                3u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::Down,
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_NO_KEY_BYTE,
                0u8,
                0u8,
                4u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::NoKey,
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_DESK_RESET_BYTE,
                0u8,
                0u8,
                5u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::DeskReset,
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                0u8,
                0u8,
                7u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::One(0.0),
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_TWO_BYTE,
                0u8,
                0u8,
                8u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::Two(0.0),
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_THREE_BYTE,
                0u8,
                0u8,
                9u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::Three(0.0),
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                138u8,
                2u8,
                147u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::One(65.0),
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_TWO_BYTE,
                138u8,
                2u8,
                148u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::Two(65.0),
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_THREE_BYTE,
                138u8,
                2u8,
                149u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::Three(65.0),
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                143u8,
                2u8,
                152u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::One(65.5),
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                232u8,
                3u8,
                242u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::One(100.0),
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                253u8,
                2u8,
                6u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::One(76.5),
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                2u8,
                3u8,
                12u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::One(77.0),
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                252u8,
                3u8,
                6u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::One(102.0),
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                1u8,
                4u8,
                12u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::One(102.5),
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                15u8,
                5u8,
                27u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::One(129.5),
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_RESET_ONE_BYTE,
                0u8,
                0u8,
                11u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::ResetOne,
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_RESET_TWO_BYTE,
                0u8,
                0u8,
                12u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::ResetTwo,
        );

        assert_eq!(
            PanelToDeskMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_RESET_THREE_BYTE,
                0u8,
                0u8,
                13u8,
                DATA_FRAME_END_BYTE
            ]),
            PanelToDeskMessage::ResetThree,
        );
    }

    #[test]
    fn test_panel_to_desk_message_as_frame() {
        assert_eq!(
            PanelToDeskMessage::Up.as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_UP_BYTE,
                0u8,
                0u8,
                2u8,
                DATA_FRAME_END_BYTE
            ],
        );

        assert_eq!(
            PanelToDeskMessage::Down.as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_DOWN_BYTE,
                0u8,
                0u8,
                3u8,
                DATA_FRAME_END_BYTE
            ],
        );

        assert_eq!(
            PanelToDeskMessage::NoKey.as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_NO_KEY_BYTE,
                0u8,
                0u8,
                4u8,
                DATA_FRAME_END_BYTE
            ],
        );

        assert_eq!(
            PanelToDeskMessage::DeskReset.as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_DESK_RESET_BYTE,
                0u8,
                0u8,
                5u8,
                DATA_FRAME_END_BYTE
            ],
        );

        assert_eq!(
            PanelToDeskMessage::Unknown(99u8, 64u8, 254u8, 1u8, 98u8).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                99u8,
                64u8,
                254u8,
                1u8,
                98u8,
                DATA_FRAME_END_BYTE
            ],
        );

        assert_eq!(
            PanelToDeskMessage::One(0.0).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                0u8,
                0u8,
                7u8,
                DATA_FRAME_END_BYTE
            ]
        );

        assert_eq!(
            PanelToDeskMessage::Two(0.0).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_TWO_BYTE,
                0u8,
                0u8,
                8u8,
                DATA_FRAME_END_BYTE
            ]
        );

        assert_eq!(
            PanelToDeskMessage::Three(0.0).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_THREE_BYTE,
                0u8,
                0u8,
                9u8,
                DATA_FRAME_END_BYTE
            ]
        );

        assert_eq!(
            PanelToDeskMessage::One(65.0).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                138u8,
                2u8,
                147u8,
                DATA_FRAME_END_BYTE
            ]
        );

        assert_eq!(
            PanelToDeskMessage::Two(65.0).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_TWO_BYTE,
                138u8,
                2u8,
                148u8,
                DATA_FRAME_END_BYTE
            ]
        );

        assert_eq!(
            PanelToDeskMessage::Three(65.0).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_THREE_BYTE,
                138u8,
                2u8,
                149u8,
                DATA_FRAME_END_BYTE
            ]
        );

        assert_eq!(
            PanelToDeskMessage::One(65.5).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                143u8,
                2u8,
                152u8,
                DATA_FRAME_END_BYTE
            ]
        );

        assert_eq!(
            PanelToDeskMessage::One(100.0).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                232u8,
                3u8,
                242u8,
                DATA_FRAME_END_BYTE
            ]
        );

        assert_eq!(
            PanelToDeskMessage::One(76.5).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                253u8,
                2u8,
                6u8,
                DATA_FRAME_END_BYTE
            ]
        );

        assert_eq!(
            PanelToDeskMessage::One(77.0).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                2u8,
                3u8,
                12u8,
                DATA_FRAME_END_BYTE
            ]
        );

        assert_eq!(
            PanelToDeskMessage::One(102.0).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                252u8,
                3u8,
                6u8,
                DATA_FRAME_END_BYTE
            ]
        );

        assert_eq!(
            PanelToDeskMessage::One(102.5).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                1u8,
                4u8,
                12u8,
                DATA_FRAME_END_BYTE
            ]
        );

        assert_eq!(
            PanelToDeskMessage::One(129.5).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                PANEL_TO_DESK_ONE_BYTE,
                15u8,
                5u8,
                27u8,
                DATA_FRAME_END_BYTE
            ]
        );
    }

    #[test]
    fn test_desk_to_panel_message_as_frame() {
        // TODO: test < 65.0
        // TODO: test > 129.5
        // TODO: test intervals of something other than 5mm / 0.5cm

        assert_eq!(
            DeskToPanelMessage::Height(65.0).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                DESK_TO_PANEL_HEIGHT_BYTE,
                0u8,
                0u8,
                1u8,
                DATA_FRAME_END_BYTE
            ],
        );

        assert_eq!(
            DeskToPanelMessage::Height(65.5).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                DESK_TO_PANEL_HEIGHT_BYTE,
                0u8,
                5u8,
                6u8,
                DATA_FRAME_END_BYTE
            ],
        );

        assert_eq!(
            DeskToPanelMessage::Height(100.0).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                DESK_TO_PANEL_HEIGHT_BYTE,
                1u8,
                94u8,
                96u8,
                DATA_FRAME_END_BYTE
            ],
        );

        assert_eq!(
            DeskToPanelMessage::Height(90.5).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                DESK_TO_PANEL_HEIGHT_BYTE,
                0u8,
                255u8,
                0u8,
                DATA_FRAME_END_BYTE
            ],
        );

        assert_eq!(
            DeskToPanelMessage::Height(91.0).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                DESK_TO_PANEL_HEIGHT_BYTE,
                1u8,
                4u8,
                6u8,
                DATA_FRAME_END_BYTE
            ],
        );

        assert_eq!(
            DeskToPanelMessage::Height(116.0).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                DESK_TO_PANEL_HEIGHT_BYTE,
                1u8,
                254u8,
                0u8,
                DATA_FRAME_END_BYTE
            ],
        );

        assert_eq!(
            DeskToPanelMessage::Height(116.5).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                DESK_TO_PANEL_HEIGHT_BYTE,
                2u8,
                3u8,
                6u8,
                DATA_FRAME_END_BYTE
            ],
        );

        assert_eq!(
            DeskToPanelMessage::Height(129.5).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                1u8,
                DESK_TO_PANEL_HEIGHT_BYTE,
                2u8,
                133u8,
                136u8,
                DATA_FRAME_END_BYTE
            ],
        );

        assert_eq!(
            DeskToPanelMessage::Unknown(99u8, 64u8, 254u8, 1u8, 98u8).as_frame(),
            vec![
                DATA_FRAME_START_BYTE,
                99u8,
                64u8,
                254u8,
                1u8,
                98u8,
                DATA_FRAME_END_BYTE
            ],
        );
    }

    #[test]
    fn test_desk_to_panel_message_from_frame() {
        assert_eq!(
            DeskToPanelMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                DESK_TO_PANEL_HEIGHT_BYTE,
                0u8,
                0u8,
                1u8,
                DATA_FRAME_END_BYTE
            ]),
            DeskToPanelMessage::Height(65.0),
        );

        assert_eq!(
            DeskToPanelMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                DESK_TO_PANEL_HEIGHT_BYTE,
                0u8,
                5u8,
                6u8,
                DATA_FRAME_END_BYTE
            ]),
            DeskToPanelMessage::Height(65.5),
        );

        assert_eq!(
            DeskToPanelMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                DESK_TO_PANEL_HEIGHT_BYTE,
                1u8,
                94u8,
                96u8,
                DATA_FRAME_END_BYTE
            ]),
            DeskToPanelMessage::Height(100.0),
        );

        assert_eq!(
            DeskToPanelMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                DESK_TO_PANEL_HEIGHT_BYTE,
                0u8,
                255u8,
                0u8,
                DATA_FRAME_END_BYTE
            ]),
            DeskToPanelMessage::Height(90.5),
        );

        assert_eq!(
            DeskToPanelMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                DESK_TO_PANEL_HEIGHT_BYTE,
                1u8,
                4u8,
                6u8,
                DATA_FRAME_END_BYTE
            ]),
            DeskToPanelMessage::Height(91.0),
        );

        assert_eq!(
            DeskToPanelMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                DESK_TO_PANEL_HEIGHT_BYTE,
                1u8,
                254u8,
                0u8,
                DATA_FRAME_END_BYTE
            ]),
            DeskToPanelMessage::Height(116.0),
        );

        assert_eq!(
            DeskToPanelMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                DESK_TO_PANEL_HEIGHT_BYTE,
                2u8,
                3u8,
                6u8,
                DATA_FRAME_END_BYTE
            ]),
            DeskToPanelMessage::Height(116.5),
        );

        assert_eq!(
            DeskToPanelMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                1u8,
                DESK_TO_PANEL_HEIGHT_BYTE,
                2u8,
                133u8,
                136u8,
                DATA_FRAME_END_BYTE
            ]),
            DeskToPanelMessage::Height(129.5),
        );

        assert_eq!(
            DeskToPanelMessage::from_frame(&vec![
                DATA_FRAME_START_BYTE,
                99u8,
                64u8,
                254u8,
                1u8,
                98u8,
                DATA_FRAME_END_BYTE
            ]),
            DeskToPanelMessage::Unknown(99u8, 64u8, 254u8, 1u8, 98u8),
        );
    }

    #[test]
    fn test_validate_frame() {
        assert!(!validate_frame(&vec![]));
        assert!(!validate_frame(&vec![0u8, 0u8]));
        assert!(!validate_frame(&vec![0u8; DATA_FRAME_SIZE]));
        assert!(!validate_frame(&vec![
            DATA_FRAME_START_BYTE,
            0u8,
            0u8,
            0u8,
            0u8,
            0u8,
            0u8
        ]));
        assert!(!validate_frame(&vec![
            0u8,
            0u8,
            0u8,
            0u8,
            0u8,
            0u8,
            DATA_FRAME_END_BYTE
        ]));

        assert!(validate_frame(&vec![
            DATA_FRAME_START_BYTE,
            0u8,
            0u8,
            0u8,
            0u8,
            0u8,
            DATA_FRAME_END_BYTE
        ]));
    }
}
