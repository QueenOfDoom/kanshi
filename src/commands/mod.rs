use crate::{Context, Error};
use poise::serenity_prelude::{Colour, CreateEmbed};
use poise::CreateReply;
use regex::Regex;
use semver::{Version, VersionReq};
use std::collections::BTreeMap;
use std::fs;
use log::debug;

#[poise::command(prefix_command, slash_command)]
pub async fn changelog(
    ctx: Context<'_>,
    #[description = "Version to the show the changelog for"]
    version: Option<String>
) -> Result<(), Error> {
    let result = fs::read("CHANGELOG.md").expect("Could not read the Changelog.");
    let changelog = String::from_utf8(result).expect("Should be able to parse the Changelog.");

    let header_index = changelog.find("## [").expect("A version section has to be in the Changelog.");
    let (header, rest) = changelog.split_at(header_index);
    let header = header.trim();
    let regex = Regex::new(r"(?<pre>\S)\r?\n(?<post>\S)").expect("Valid Regex!");
    let header= regex.replace_all(header, "$pre $post").to_string();
    let regex = Regex::new(r"\r?\n(?<newlines>(?:\r?\n)+)").expect("Valid Regex!");
    let header = regex.replace_all(&header, "$newlines").to_string();

    match version {
        None => {
            let body = format!("{}\n\n{}", header, rest);
            let see_more = "... [See More](https://github.com/QueenOfDoom/kanshi/blob/master/CHANGELOG.md)";
            let total_length = body.len() + see_more.len();
            let final_body = if body.len() > 4096 {
                let to_remove = total_length - 4096;
                let mut trimmed_rest = &rest[..(rest.len() - to_remove)];
                if let Some(last_space) = trimmed_rest.rfind(' ') {
                    trimmed_rest = &trimmed_rest[..last_space];
                }
                format!("{}\n\n{}{}", header, trimmed_rest, see_more)
            } else {
                body
            };

            // Create and send the reply with the final description
            let reply = CreateReply::default().embed(
                CreateEmbed::default()
                    .color(Colour::BLURPLE)
                    .description(final_body)
            );
            ctx.send(reply).await?;
        }
        Some(version_spec) => {
            let latest_version_start = rest.find("[").expect("A version-bracket (open) must be included.");
            let latest_version_end = rest.find("]").expect("A version-bracket (closed) must be included.");
            let version = &rest[(latest_version_start + 1)..latest_version_end];

            let footer_indicator = format!("[{}]: ", version);
            let footer_index = rest.find(&footer_indicator).expect("A version footnote must exist!");
            let (version_section, footer) = rest.split_at(footer_index);

            let version_regex = Regex::new(r"## \[(?<version>[^]]+)]").expect("Valid regex!");
            let mut version_map: BTreeMap<Version, String> = BTreeMap::new();
            let mut last_version: Option<Version> = None;

            let captures: Vec<_> = version_regex.captures_iter(version_section).collect();

            for cap in captures.into_iter().rev() {
                let version_str = &cap["version"];
                let version = if version_str == "Unreleased" {
                    match last_version {
                        None => Version::new(0, 1, 0),
                        Some(ref v) => Version::new(v.major, v.minor + 1, 0)
                    }
                } else { Version::parse(version_str).expect("Valid semver version") };
                last_version = Some(version.clone());

                let start_index = cap.get(0).unwrap().start();
                let end_index = version_section[start_index..].find("\n## [")
                    .unwrap_or(version_section.len());

                let section = &version_section[start_index..end_index];
                version_map.insert(version, section.trim().to_string());
            }

            let version_spec = version_spec.trim_start_matches("v");
            
            match VersionReq::parse(version_spec) {
                Ok(requirement) => {
                    let footer = footer.lines();
                    let mut valid_versions = vec![];
                    let mut footnotes = vec![];
                    for ((version, corpus), footnote) in version_map.iter().rev().zip(footer) {
                        if requirement.matches(version) {
                            valid_versions.push(corpus.to_string());
                            footnotes.push(footnote);
                        }
                    }
                    
                    let body = if valid_versions.is_empty() {
                        ":mag_right: Version Requirement didn't match any existing version.".to_string()
                    } else { valid_versions.join("\n\n") };
                    
                    let footnotes = footnotes.join("\n");
                    
                    let reply = CreateReply::default().embed(CreateEmbed::default()
                        .color(Colour::BLURPLE)
                        .description(format!("{}\n\n{}\n\n{}", header, body, footnotes)));
                    ctx.send(reply).await?;
                    
                }
                Err(e) => {
                    let reply = CreateReply::default()
                        .content(format!(":octagonal_sign: Invalid Version Requirement: {}", version_spec));
                    let author = ctx.author();
                    debug!("{} ({}) entered invalid version requirement and caused: {}", author.name, author.id, e);
                    ctx.send(reply).await?;
                }
            }
        }
    }
    Ok(())
}