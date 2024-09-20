use crate::util::discord::build_changelog;

//noinspection SpellCheckingInspection
#[test]
fn test_limit_content_and_see_more() {
    use crate::util::discord::limit_content_and_see_more;

    let hello = limit_content_and_see_more(
        6,
        vec![vec!["Hello", "World"]],
        |mut v| v.next().unwrap().join(" "),
        None,
    )
    .unwrap();

    assert_eq!("Hello", hello);

    let changelog_components = vec![
        vec!["# Changelog\n\nAll notable changes to the 'Kanshi' project will be documented in this file.\nThe format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) as well as [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)."],
        vec![
            "## [Unreleased]\n\n### Added\n\n- Changelog Command with SemVer query support\n- Optimize Database queries by using a Connection Pool via `r2d2`\n- Registry of edited messages from when the bot wasn't online yet\n\n### Changed\n\n- Migrated from `rusqlite` to `diesel` for the sake of having a proper ORM\n\n### Fixed\n\n- Migrated from `dotenv` to `dotenvy` ([RUSTSEC-2021-0141](https://rustsec.org/advisories/RUSTSEC-2021-0141.html))\n- Patched [0.1.0] tag date.",
            "## [0.1.0] - 2024-09-19\n\n### Added\n\n- Discord message edit and deletion logging via Discord Bot and Embeds\n- Preservation of deleted messages within a SQLite Database\n- Console & File logging of Discord Events"
        ],
        vec!["[Unreleased]: https://github.com/QueenOfDoom/kanshi/compare/v0.1.0...HEAD", "[0.1.0]: https://github.com/QueenOfDoom/kanshi/releases/tag/v0.1.0"]
    ];
    let changelog_url = Some((
        "https://github.com/QueenOfDoom/kanshi/blob/master/CHANGELOG.md",
        2,
    ));

    let tiny_changelog = limit_content_and_see_more(
        256,
        changelog_components.clone(),
        build_changelog,
        changelog_url,
    );
    assert!(tiny_changelog.is_ok());
    assert_eq!(tiny_changelog.unwrap().len(), 82);

    let changelog = limit_content_and_see_more(
        1024,
        changelog_components.clone(),
        build_changelog,
        changelog_url,
    );
    assert!(changelog.is_ok());
    assert_eq!(changelog.unwrap().len(), 945);

    let full_changelog =
        limit_content_and_see_more(4096, changelog_components, build_changelog, changelog_url);
    assert!(full_changelog.is_ok());
    assert_eq!(full_changelog.unwrap().len(), 1144);
}
