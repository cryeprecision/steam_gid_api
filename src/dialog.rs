use std::str::FromStr;

use dialoguer::theme::ColorfulTheme;

use crate::steam::GroupIdentifier;

pub async fn prompt_input() -> GroupIdentifier {
    let input = tokio::task::spawn_blocking(|| {
        dialoguer::Input::with_theme(&ColorfulTheme::default())
            .allow_empty(false)
            .report(true)
            .with_prompt("Enter Identifier")
            .validate_with(|input: &String| GroupIdentifier::from_str(input).map(|_| ()))
            .interact_text()
            .unwrap()
    })
    .await
    .unwrap();

    GroupIdentifier::from_str(&input).unwrap()
}
