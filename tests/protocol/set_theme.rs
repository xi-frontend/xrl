use std::io;

use xrl::client::ClientExt;
use xrl::protocol::{Message, ThemeChanged, ThemeSettings, XiNotification};
use xrl::TestClient;

#[tokio::test]
async fn set_theme() -> io::Result<()> {
    let mut client = TestClient::embeded().await?;
    let theme_change = ThemeChanged {
        name: "InspiredGitHub".into(),
        theme: ThemeSettings::default(),
    };
    let expected = Message::Notification(XiNotification::ThemeChanged(theme_change));
    client.set_theme("InspiredGitHub").await?;
    client.check_responses(None, expected).await?;
    Ok(())
}

#[tokio::test]
async fn set_theme_invalid() -> io::Result<()> {
    let mut client = TestClient::embeded().await?;
    let expected = Message::Error("Error from xi".into());
    client.set_theme("non_theme").await?;
    if let Ok(_) = client.check_responses(None, expected).await {
        panic!("TestClient::check_responses should have errored");
    }
    Ok(())
}
