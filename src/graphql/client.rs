use graphql_client::GraphQLQuery;
use reqwest::Client;
use std::fmt;

/// Errors returned when calling the workforce_analytics GraphQL API.
#[derive(Debug)]
pub enum ApiError {
    /// Network or deserialization failure from reqwest
    Request(reqwest::Error),
    /// The API responded, but with GraphQL errors (e.g. permission denied)
    GraphQL(Vec<graphql_client::Error>),
    /// The API responded without errors but also without data
    MissingData,
    /// The form's CSRF token was stale, so the mutation was never sent
    /// (see `handlers::utility::guarded`).
    StaleForm,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::Request(e) => write!(f, "API request failed: {}", e),
            ApiError::GraphQL(errors) => {
                let messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
                write!(f, "{}", messages.join("; "))
            }
            ApiError::MissingData => write!(f, "API response contained no data"),
            ApiError::StaleForm => write!(f, "Form token was stale; nothing was sent"),
        }
    }
}

impl std::error::Error for ApiError {}

/// Form fields whose values are unique in the database. A unique-constraint
/// violation names its column (`org_tiers_org_name_en_key`), which is how an
/// error finds its way back to the field that caused it.
const UNIQUE_FIELDS: [&str; 11] = [
    "name_en", "name_fr", "title_en", "title_fr", "acronym_en", "acronym_fr",
    "email", "phone", "peoplesoft_id", "orcid_id", "reference_number",
];

impl ApiError {
    fn messages(&self) -> String {
        match self {
            ApiError::GraphQL(errors) => errors.iter().map(|e| e.message.as_str()).collect::<Vec<_>>().join("; "),
            _ => String::new(),
        }
    }

    /// What to tell the person who submitted the form, in their language.
    /// Database and transport failures are replaced by a plain explanation
    /// (the raw text is logged by `post_graphql`); the API's own validation
    /// messages are already written for people and pass through.
    pub fn user_message(&self, lang: &str) -> String {
        let raw = self.messages();
        let text = match self {
            ApiError::Request(_) => crate::by_lang(lang,
                "The server couldn't be reached. Nothing was saved; try again.",
                "Le serveur est injoignable. Rien n'a été enregistré; réessayez."),
            ApiError::MissingData => crate::by_lang(lang,
                "The server returned an empty response. Nothing was saved; try again.",
                "Le serveur a renvoyé une réponse vide. Rien n'a été enregistré; réessayez."),
            ApiError::StaleForm => crate::by_lang(lang,
                "Your session expired before this was saved. Check the form and press Save again.",
                "Votre session a expiré avant l'enregistrement. Vérifiez le formulaire et enregistrez de nouveau."),
            ApiError::GraphQL(_) if raw.contains("violates unique constraint") => crate::by_lang(lang,
                "Another record already uses that value. Change the highlighted field and save again.",
                "Un autre enregistrement utilise déjà cette valeur. Modifiez le champ indiqué et enregistrez de nouveau."),
            ApiError::GraphQL(_) if raw.contains("violates foreign key constraint") => crate::by_lang(lang,
                "Other records depend on this one, so the change can't be made.",
                "D'autres enregistrements dépendent de celui-ci; la modification est impossible."),
            ApiError::GraphQL(_) if raw.contains("db connection") || raw.contains("timed out") => crate::by_lang(lang,
                "The server is busy. Nothing was saved; try again in a moment.",
                "Le serveur est occupé. Rien n'a été enregistré; réessayez dans un instant."),
            ApiError::GraphQL(_) if raw.contains("Access denied") || raw.contains("Token") || raw.contains("Signature") => crate::by_lang(lang,
                "You don't have permission to do that, or your sign-in has expired.",
                "Vous n'avez pas l'autorisation, ou votre session a expiré."),
            ApiError::GraphQL(_) => return raw,
        };
        text.to_string()
    }

    /// The form field this error is about, with the message to show beside
    /// it. Only uniqueness violations name a field today.
    pub fn field_error(&self, lang: &str) -> Option<(&'static str, &'static str)> {
        let raw = self.messages();
        let constraint = raw.split("unique constraint").nth(1)?;
        let field = UNIQUE_FIELDS.iter().find(|f| constraint.contains(*f))?;
        Some((field, crate::by_lang(lang, "Already in use — choose a different value.", "Déjà utilisé — choisissez une autre valeur.")))
    }
}


impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        ApiError::Request(e)
    }
}

/// Send a GraphQL query or mutation to the API and return its data.
///
/// All API calls should go through this function: it attaches the JWT as
/// `Authorization: Bearer <token>` (the header the API actually validates)
/// and surfaces GraphQL errors instead of panicking on missing data.
pub async fn post_graphql<Q: GraphQLQuery>(
    client: &Client,
    api_url: &str,
    bearer: &str,
    variables: Q::Variables,
) -> Result<Q::ResponseData, ApiError> {
    let request_body = Q::build_query(variables);

    let mut request = client.post(api_url).json(&request_body);

    if !bearer.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", bearer));
    }

    let response_body: graphql_client::Response<Q::ResponseData> =
        request.send().await?.json().await?;

    if let Some(errors) = response_body.errors {
        if !errors.is_empty() {
            println!("GraphQL errors: {:?}", &errors);
            return Err(ApiError::GraphQL(errors));
        }
    }

    response_body.data.ok_or(ApiError::MissingData)
}

#[cfg(test)]
mod tests {
    use super::ApiError;

    fn graphql(message: &str) -> ApiError {
        ApiError::GraphQL(vec![graphql_client::Error {
            message: message.to_string(), locations: None, path: None, extensions: None,
        }])
    }

    #[test]
    fn database_errors_become_plain_language_with_a_field() {
        let e = graphql("duplicate key value violates unique constraint \"org_tiers_org_name_en_key\"");
        assert!(!e.user_message("fr").contains("duplicate key"));
        assert!(e.user_message("fr").starts_with("Un autre enregistrement"));
        assert_eq!(e.field_error("en").map(|(f, _)| f), Some("name_en"));
    }

    #[test]
    fn api_validation_messages_pass_through_without_a_field() {
        let e = graphql("A role must report to a more senior position");
        assert_eq!(e.user_message("en"), "A role must report to a more senior position");
        assert_eq!(e.field_error("en"), None);
    }
}
