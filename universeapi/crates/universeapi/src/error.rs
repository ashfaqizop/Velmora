use crate::model::ProviderId;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("network error for {url}: {source}")]
    Network {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("HTTP {status} from {url}")]
    Http { status: u16, url: String },

    #[error("blocked by anti-bot protection at {url}")]
    Blocked { url: String },

    #[error("could not parse {what}: {detail}")]
    Parse { what: &'static str, detail: String },

    #[error("not found: {0}")]
    NotFound(String),

    #[error("unsupported: {0}")]
    Unsupported(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("transport: {0}")]
    Transport(String),

    #[error("torrent: {0}")]
    Torrent(String),

    #[error("[{provider}] {message}")]
    Provider { provider: ProviderId, message: String },
}

impl Error {
    pub fn parse(what: &'static str, detail: impl Into<String>) -> Self {
        Self::Parse { what, detail: detail.into() }
    }

    pub fn provider(provider: ProviderId, message: impl Into<String>) -> Self {
        Self::Provider { provider, message: message.into() }
    }

    /// Whether retrying the same request later could plausibly succeed.
    pub fn is_transient(&self) -> bool {
        match self {
            Self::Network { .. } | Self::Transport(_) => true,
            Self::Http { status, .. } => matches!(status, 408 | 425 | 429 | 500..=599),
            _ => false,
        }
    }
}
