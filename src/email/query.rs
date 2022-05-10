use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::core::{query, set::from_timestamp};

#[derive(Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum Filter {
    InMailbox {
        #[serde(rename = "inMailbox")]
        value: String,
    },
    InMailboxOtherThan {
        #[serde(rename = "inMailboxOtherThan")]
        value: Vec<String>,
    },
    Before {
        #[serde(rename = "before")]
        value: DateTime<Utc>,
    },
    After {
        #[serde(rename = "after")]
        value: DateTime<Utc>,
    },
    MinSize {
        #[serde(rename = "minSize")]
        value: u32,
    },
    MaxSize {
        #[serde(rename = "maxSize")]
        value: u32,
    },
    AllInThreadHaveKeyword {
        #[serde(rename = "allInThreadHaveKeyword")]
        value: String,
    },
    SomeInThreadHaveKeyword {
        #[serde(rename = "someInThreadHaveKeyword")]
        value: String,
    },
    NoneInThreadHaveKeyword {
        #[serde(rename = "noneInThreadHaveKeyword")]
        value: String,
    },
    HasKeyword {
        #[serde(rename = "hasKeyword")]
        value: String,
    },
    NotKeyword {
        #[serde(rename = "notKeyword")]
        value: String,
    },
    HasAttachment {
        #[serde(rename = "hasAttachment")]
        value: bool,
    },
    Text {
        #[serde(rename = "text")]
        value: String,
    },
    From {
        #[serde(rename = "from")]
        value: String,
    },
    To {
        #[serde(rename = "to")]
        value: String,
    },
    Cc {
        #[serde(rename = "cc")]
        value: String,
    },
    Bcc {
        #[serde(rename = "bcc")]
        value: String,
    },
    Subject {
        #[serde(rename = "subject")]
        value: String,
    },
    Body {
        #[serde(rename = "body")]
        value: String,
    },
    Header {
        #[serde(rename = "header")]
        value: Vec<String>,
    },
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "property")]
pub enum Comparator {
    #[serde(rename = "receivedAt")]
    ReceivedAt,
    #[serde(rename = "size")]
    Size,
    #[serde(rename = "from")]
    From,
    #[serde(rename = "to")]
    To,
    #[serde(rename = "subject")]
    Subject,
    #[serde(rename = "sentAt")]
    SentAt,
    #[serde(rename = "hasKeyword")]
    HasKeyword { keyword: String },
    #[serde(rename = "allInThreadHaveKeyword")]
    AllInThreadHaveKeyword { keyword: String },
    #[serde(rename = "someInThreadHaveKeyword")]
    SomeInThreadHaveKeyword { keyword: String },
}

impl Filter {
    pub fn in_mailbox(value: impl Into<String>) -> Self {
        Filter::InMailbox {
            value: value.into(),
        }
    }

    pub fn in_mailbox_other_than<U, V>(value: U) -> Self
    where
        U: IntoIterator<Item = V>,
        V: Into<String>,
    {
        Filter::InMailboxOtherThan {
            value: value.into_iter().map(|v| v.into()).collect(),
        }
    }

    pub fn before(value: i64) -> Self {
        Filter::Before {
            value: from_timestamp(value),
        }
    }

    pub fn after(value: i64) -> Self {
        Filter::After {
            value: from_timestamp(value),
        }
    }

    pub fn min_size(value: u32) -> Self {
        Filter::MinSize { value }
    }

    pub fn max_size(value: u32) -> Self {
        Filter::MaxSize { value }
    }

    pub fn all_in_thread_have_keyword(value: impl Into<String>) -> Self {
        Filter::AllInThreadHaveKeyword {
            value: value.into(),
        }
    }

    pub fn some_in_thread_have_keyword(value: impl Into<String>) -> Self {
        Filter::SomeInThreadHaveKeyword {
            value: value.into(),
        }
    }

    pub fn none_in_thread_have_keyword(value: impl Into<String>) -> Self {
        Filter::NoneInThreadHaveKeyword {
            value: value.into(),
        }
    }

    pub fn has_keyword(value: impl Into<String>) -> Self {
        Filter::HasKeyword {
            value: value.into(),
        }
    }

    pub fn not_keyword(value: impl Into<String>) -> Self {
        Filter::NotKeyword {
            value: value.into(),
        }
    }

    pub fn has_attachment(value: bool) -> Self {
        Filter::HasAttachment { value }
    }

    pub fn text(value: impl Into<String>) -> Self {
        Filter::Text {
            value: value.into(),
        }
    }

    pub fn from(value: impl Into<String>) -> Self {
        Filter::From {
            value: value.into(),
        }
    }

    pub fn to(value: impl Into<String>) -> Self {
        Filter::To {
            value: value.into(),
        }
    }

    pub fn cc(value: impl Into<String>) -> Self {
        Filter::Cc {
            value: value.into(),
        }
    }

    pub fn bcc(value: impl Into<String>) -> Self {
        Filter::Bcc {
            value: value.into(),
        }
    }

    pub fn subject(value: impl Into<String>) -> Self {
        Filter::Subject {
            value: value.into(),
        }
    }

    pub fn body(value: impl Into<String>) -> Self {
        Filter::Body {
            value: value.into(),
        }
    }

    pub fn header<U, V>(value: U) -> Self
    where
        U: IntoIterator<Item = V>,
        V: Into<String>,
    {
        Filter::Header {
            value: value.into_iter().map(|v| v.into()).collect(),
        }
    }
}

impl Comparator {
    pub fn received_at() -> query::Comparator<Comparator> {
        query::Comparator::new(Comparator::ReceivedAt)
    }

    pub fn size() -> query::Comparator<Comparator> {
        query::Comparator::new(Comparator::Size)
    }

    pub fn from() -> query::Comparator<Comparator> {
        query::Comparator::new(Comparator::From)
    }

    pub fn to() -> query::Comparator<Comparator> {
        query::Comparator::new(Comparator::To)
    }

    pub fn subject() -> query::Comparator<Comparator> {
        query::Comparator::new(Comparator::Subject)
    }

    pub fn sent_at() -> query::Comparator<Comparator> {
        query::Comparator::new(Comparator::SentAt)
    }

    pub fn has_keyword(keyword: impl Into<String>) -> query::Comparator<Comparator> {
        query::Comparator::new(Comparator::HasKeyword {
            keyword: keyword.into(),
        })
    }

    pub fn all_in_thread_have_keyword(keyword: impl Into<String>) -> query::Comparator<Comparator> {
        query::Comparator::new(Comparator::AllInThreadHaveKeyword {
            keyword: keyword.into(),
        })
    }

    pub fn some_in_thread_have_keyword(
        keyword: impl Into<String>,
    ) -> query::Comparator<Comparator> {
        query::Comparator::new(Comparator::SomeInThreadHaveKeyword {
            keyword: keyword.into(),
        })
    }
}
