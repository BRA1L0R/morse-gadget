use core::{fmt::Display, mem::MaybeUninit};

use alloc::boxed::Box;
use heapless::String;

#[derive(Debug)]
pub enum From {
    You,
    Other,
    /// system messages for service information
    System,
}

impl Display for From {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            From::You => f.write_str("YOU"),
            From::System => f.write_str("[>]"),
            From::Other => f.write_str("OTH"),
        }
    }
}

#[derive(Debug)]
pub struct ChatMessage {
    pub from: From,
    pub text: String<16>,
}

/// Circular buffer for messages
pub struct ChatLog {
    log: Box<[MaybeUninit<ChatMessage>; 8]>,

    index: usize,
    used: usize,
}

impl ChatLog {
    pub fn new() -> Self {
        Self {
            log: Box::new(unsafe { MaybeUninit::uninit().assume_init() }),
            index: 0,
            used: 0,
        }
    }

    // pub fn initialized(&self) -> &[ChatMessage] {
    //     unsafe { core::mem::transmute(&self.log[..self.used]) }
    // }

    pub fn messages(&self) -> impl Iterator<Item = &ChatMessage> {
        self.log[self.index..self.used]
            .iter()
            .chain(self.log[..self.index].iter())
            .map(|uninit| unsafe { uninit.assume_init_ref() })
            .rev()
    }

    pub fn push_message(&mut self, from: From, text: impl Into<String<16>>) {
        let message = ChatMessage {
            from,
            text: text.into(),
        };

        self.log[self.index] = MaybeUninit::new(message);
        self.index = (self.index + 1) % self.log.len();

        let used = (self.used + 1) % (self.log.len() + 1);
        if used > self.used {
            self.used = used;
        }
    }
}
