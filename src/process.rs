use crate::protocol::{Frame, Parser};

pub trait Process {
    fn process(self) -> Vec<Frame>;
}

impl Process for Parser<'_> {
    fn process(self) -> Vec<Frame> {
        let mut r = vec![];
        for f in self.frames {
            match f {
                Frame::Array(v) => {
                    for f in v {
                        match f {
                            Frame::Simple(s) => {
                                if s.to_ascii_uppercase() == "PING" {
                                    r.push(Frame::Simple("PONG".into()))
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        tracing::trace!("returning vector {:?}", r);
        r
    }
}
