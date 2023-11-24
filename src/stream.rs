use async_trait::async_trait;
use futures_util::{stream::SplitStream, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::WebSocketStream;

#[async_trait]
pub trait GetNextTrait {
    async fn get_next_text(&mut self) -> Option<String>;
}

#[async_trait]
impl<T> GetNextTrait for SplitStream<WebSocketStream<T>>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    async fn get_next_text(&mut self) -> Option<String> {
        self.next()
            .await
            .and_then(|result| result.ok())
            .and_then(|message| message.into_text().ok())
    }
}
