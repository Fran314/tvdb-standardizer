use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum Urgency {
    Info,
    Warning,
    Error,
}
#[derive(Debug)]
pub struct Message {
    pub timestamp: Instant,
    pub urgency: Urgency,
    pub content: String,
}

impl Message {
    pub fn new(content: String, urgency: Urgency) -> Self {
        Self {
            timestamp: Instant::now(),
            urgency,
            content,
        }
    }

    pub fn info(content: String) -> Self {
        Self::new(content, Urgency::Info)
    }
    pub fn warning(content: String) -> Self {
        Self::new(content, Urgency::Warning)
    }
    pub fn error(content: String) -> Self {
        Self::new(content, Urgency::Error)
    }

    pub fn err_to_warn(self) -> Self {
        match self {
            Self {
                timestamp,
                content,
                urgency: Urgency::Error,
            } => Self {
                timestamp,
                content,
                urgency: Urgency::Warning,
            },
            _ => self,
        }
    }

    pub fn warn_to_err(self) -> Self {
        match self {
            Self {
                timestamp,
                content,
                urgency: Urgency::Warning,
            } => Self {
                timestamp,
                content,
                urgency: Urgency::Error,
            },
            _ => self,
        }
    }

    pub fn has_expired(&self) -> bool {
        Instant::now().duration_since(self.timestamp) > Duration::from_secs(5)
    }
}

#[derive(Debug)]
pub struct WithMessages<T> {
    pub inner: T,
    pub messages: Vec<Message>,
}

impl<T: Default> Default for WithMessages<T> {
    fn default() -> Self {
        Self {
            inner: T::default(),
            messages: Vec::new(),
        }
    }
}

impl<T: Default> WithMessages<T> {
    pub fn from_message(messages: Message) -> Self {
        Self {
            inner: T::default(),
            messages: vec![messages],
        }
    }
    pub fn from_messages(messages: Vec<Message>) -> Self {
        Self {
            inner: T::default(),
            messages,
        }
    }
}

impl<T> WithMessages<T> {
    pub fn from_value(inner: T) -> Self {
        Self {
            inner,
            messages: Vec::new(),
        }
    }
    pub fn new(inner: T, messages: Vec<Message>) -> Self {
        Self { inner, messages }
    }

    pub fn append_messages(self, global_messages: &mut Vec<Message>) -> T {
        let Self {
            inner,
            mut messages,
        } = self;
        global_messages.append(&mut messages);
        inner
    }

    pub fn destructure(self) -> (T, Vec<Message>) {
        let Self { inner, messages } = self;
        (inner, messages)
    }

    pub fn map<B, F>(self, mut f: F) -> WithMessages<B>
    where
        F: FnMut(T) -> B,
    {
        let WithMessages::<T> { inner, messages } = self;
        WithMessages::<B> {
            inner: f(inner),
            messages,
        }
    }
}
