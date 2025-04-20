# apnoxide

Apple Push Notification Service for Rust built on `reqwest`

## Features

- Handles signature renewing for Apple's guidelines.
- Intuitive structs, easy to build payload.

## Examples

```rust
fn main() {
    // currently supports PKCS #8 only
    let key = fs::read_to_string("/path/to/private_key.p8").unwrap();

    // initialize apn client
    let cfg =
        APNClientConfig::new("TEAM_ID", "KEY_ID", &key, Endpoint::production()).unwrap();
    let mut client = APNClient::new(cfg).unwrap();

    // build payload
    let payload = Payload {
        aps: Notification {
            alert: Some(Alert::Full {
                title: Some(Title::Normal("Title".to_string())),
                subtitle: Some(Subtitle::Localized {
                    key: "SUBTITLE_KEY".to_string(),
                    args: None,
                }),
                body: None,
                launch_image: None,
            }),
            sound: Some(Sound::Critical {
                critical: Some(true),
                name: None,
                volume: None,
            }),
            mutable_content: Some(true),
            interruption_level: Some(InterruptionLevel::TimeSensitive),
            ..Notification::default()
        },
        custom: None,
    };

    // build options
    let option = PushOption {
        topic: "com.example.topic",
        ..PushOption::default()
    };

    // push notification
    let res = client
        .push(
            &payload,
            "device_token",
            option,
        )
        .await;

    // handle error
    match res {
        Ok(_) => {}
        Err(err) => {
            match err {
                APNError => {
                    println!("{:?}", err.error)
                }
                _ => {}
            }
        }
    }
}

```