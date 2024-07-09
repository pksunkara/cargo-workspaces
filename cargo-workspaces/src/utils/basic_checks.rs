use url::Url;

use crate::utils::{warn, Error, Result};

/// Performs basic checks to make sure that crate can be published.
/// Returns `Ok(())` if no problems were found, otherwise returns a list of
/// strings, each describing a problem.
///
/// This method is a simple heuristic, and if it returns `Ok(())`, it does not
/// guarantee that the crate can be published successfully.
///
/// Current list of checks is based on the [cargo reference recommendations][1].
///
/// [1]: https://doc.rust-lang.org/cargo/reference/publishing.html#before-publishing-a-new-crate
pub fn basic_checks(pkg: &cargo_metadata::Package) -> Result<()> {
    let mut problems = Vec::new();

    // Mandatory fields.
    if pkg.description.is_none() {
        problems.push("'description' field should be set".to_string());
    }
    if pkg.license.is_none() && pkg.license_file.is_none() {
        problems.push("either 'license' or 'license-file' field should be set".to_string());
    }

    // Too long description.
    const MAX_DESCRIPTION_LEN: usize = 1000;
    if pkg
        .description
        .as_ref()
        .map(|d| d.len() > MAX_DESCRIPTION_LEN)
        .unwrap_or(false)
    {
        problems.push(format!(
            "Description is too long (max {} characters)",
            MAX_DESCRIPTION_LEN
        ));
    }

    // URLs must be valid.
    validate_url(&pkg.homepage.as_deref(), "homepage", &mut problems);
    validate_url(
        &pkg.documentation.as_deref(),
        "documentation",
        &mut problems,
    );
    validate_url(&pkg.repository.as_deref(), "repository", &mut problems);

    // Keywords limit and size
    const MAX_KEYWORDS: usize = 5;
    if pkg.keywords.len() > MAX_KEYWORDS {
        problems.push(format!("Too many keywords (max {} keywords)", MAX_KEYWORDS));
    }

    const MAX_KEYWORD_LEN: usize = 20;
    for kw in pkg.keywords.iter() {
        if kw.len() > MAX_KEYWORD_LEN {
            problems.push(format!(
                "Keyword is too long (max {} characters): {}",
                MAX_KEYWORD_LEN, kw
            ));
        } else if !valid_keyword(kw) {
            problems.push(format!("Keyword contains invalid characters: {}", kw));
        }
    }

    if problems.is_empty() {
        Ok(())
    } else {
        let name = pkg.name.clone();
        warn!("  basic checks failed", name);
        for problem in problems {
            warn!("  - {}", problem);
        }
        Err(Error::Publish(name))
    }
}

// Adapted from:
// https://github.com/rust-lang/crates.io/blob/d507a12560ab923c2a1a061e5365fe6b1f1293a8/src/models/keyword.rs#L56
fn valid_keyword(keyword: &str) -> bool {
    let mut chars = keyword.chars();
    let first = match chars.next() {
        None => return false,
        Some(c) => c,
    };
    first.is_ascii_alphanumeric()
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '+')
}

// Adapted from:
// https://github.com/rust-lang/crates.io/blob/d507a12560ab923c2a1a061e5365fe6b1f1293a8/src/controllers/krate/publish.rs#L233
fn validate_url(url: &Option<&str>, field: &str, problems: &mut Vec<String>) {
    let Some(url) = url else {
        return;
    };

    // Manually check the string, as `Url::parse` may normalize relative URLs
    // making it difficult to ensure that both slashes are present.
    if !url.starts_with("http://") && !url.starts_with("https://") {
        problems.push(format!(
            "URL for field `{field}` must begin with http:// or https:// (url: {url})"
        ));
    }

    // Ensure the entire URL parses as well
    if Url::parse(url).is_err() {
        problems.push(format!("`{field}` is not a valid url: `{url}`"));
    }
}
