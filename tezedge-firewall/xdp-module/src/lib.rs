#![no_std]

#[derive(Debug, Clone)]
pub struct EndpointPair {
    pub remote: Endpoint,
    pub local: Endpoint,
}

// TODO: ipv6
#[derive(Clone)]
pub struct Endpoint {
    pub ipv4: [u8; 4],
    pub port: [u8; 2],
}

#[derive(Debug, Clone)]
pub struct Event {
    pub pair: EndpointPair,
    pub event: EventInner,
}

#[derive(Clone)]
#[repr(u32)]
pub enum EventInner {
    ReceivedPow([u8; 56]),
    NotEnoughBytesForPow,
    BlockedAlreadyConnected {
        already_connected: Endpoint,
        try_connect: Endpoint,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlockingReason {
    NoBlocking,
    CommandLineArgument,
    BadProofOfWork,
    AlreadyConnected,
    EventFromTezedge,
}

bitflags::bitflags! {
    pub struct Status: u32 {
        const BLOCKED = 0b00000000_00000000_00000000_00000001;
        const POW_SENT = 0b00000000_00000000_00000000_00000010;
    }
}

mod implementations {
    use core::{
        fmt,
        convert::{TryFrom, TryInto},
    };
    use super::{EndpointPair, Endpoint, EventInner};

    impl From<EndpointPair> for [u8; 12] {
        fn from(v: EndpointPair) -> Self {
            let mut r = [0; 12];
            r[0..6].clone_from_slice(<[u8; 6]>::from(v.local).as_ref());
            r[6..12].clone_from_slice(<[u8; 6]>::from(v.remote).as_ref());
            r
        }
    }

    impl From<[u8; 12]> for EndpointPair {
        fn from(r: [u8; 12]) -> Self {
            EndpointPair {
                local: <[u8; 6]>::try_from(&r[0..6]).unwrap().into(),
                remote: <[u8; 6]>::try_from(&r[6..12]).unwrap().into(),
            }
        }
    }

    impl fmt::Debug for Endpoint {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let ip = self.ipv4;
            let port = u16::from_be_bytes(self.port);
            write!(f, "{}.{}.{}.{}:{}", ip[0], ip[1], ip[2], ip[3], port)
        }
    }

    impl From<Endpoint> for [u8; 6] {
        fn from(v: Endpoint) -> Self {
            let mut r = [0; 6];
            r[0..4].clone_from_slice(v.ipv4.as_ref());
            r[4..6].clone_from_slice(v.port.as_ref());
            r
        }
    }

    impl From<[u8; 6]> for Endpoint {
        fn from(r: [u8; 6]) -> Self {
            Endpoint {
                ipv4: TryFrom::try_from(&r[0..4]).unwrap(),
                port: TryFrom::try_from(&r[4..6]).unwrap(),
            }
        }
    }

    impl fmt::Debug for EventInner {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                &EventInner::ReceivedPow(ref b) => b
                    .as_ref()
                    .into_iter()
                    .fold(&mut f.debug_tuple("ReceivedPow"), |d, b| d.field(b))
                    .finish(),
                &EventInner::NotEnoughBytesForPow => f.debug_tuple("NotEnoughBytesForPow").finish(),
                &EventInner::BlockedAlreadyConnected {
                    ref already_connected,
                    ref try_connect,
                } => f
                    .debug_struct("BlockedAlreadyConnected")
                    .field("already_connected", already_connected)
                    .field("try_connect", try_connect)
                    .finish(),
            }
        }
    }

    impl From<EventInner> for [u8; 60] {
        fn from(v: EventInner) -> Self {
            let mut r = [0; 60];
            match v {
                EventInner::ReceivedPow(b) => {
                    r[0..4].clone_from_slice(0u32.to_le_bytes().as_ref());
                    r[4..].clone_from_slice(b.as_ref());
                    r
                },
                EventInner::NotEnoughBytesForPow => {
                    r[0..4].clone_from_slice(1u32.to_le_bytes().as_ref());
                    r
                },
                EventInner::BlockedAlreadyConnected {
                    already_connected,
                    try_connect,
                } => {
                    r[0..4].clone_from_slice(2u32.to_le_bytes().as_ref());
                    r[4..10].clone_from_slice(<[u8; 6]>::from(already_connected).as_ref());
                    r[10..16].clone_from_slice(<[u8; 6]>::from(try_connect).as_ref());
                    r
                },
            }
        }
    }

    impl From<[u8; 60]> for EventInner {
        fn from(r: [u8; 60]) -> Self {
            let d = u32::from_le_bytes(r[0..4].try_into().unwrap());
            match d {
                0 => {
                    let mut b = [0; 56];
                    b.clone_from_slice(&r[4..]);
                    EventInner::ReceivedPow(b)
                },
                1 => EventInner::NotEnoughBytesForPow,
                2 => {
                    let already_connected = <[u8; 6]>::try_from(&r[4..10]).unwrap().into();
                    let try_connect = <[u8; 6]>::try_from(&r[10..16]).unwrap().into();
                    EventInner::BlockedAlreadyConnected {
                        already_connected,
                        try_connect,
                    }
                },
                _ => panic!(),
            }
        }
    }
}
