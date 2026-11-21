//! Where updates come from: WoodSketch's own GitHub Releases, read over HTTPS that trusts the
//! roots the computer trusts (`woodsketch_ui_egui::net`).
//!
//! Velopack has a `GithubSource` that makes these same two requests, and it is what WoodSketch
//! used, but it builds its HTTPS client with only the roots compiled into the binary and offers
//! no way to change them (`HttpOptions` carries headers and a timeout, nothing else). On a
//! machine whose HTTPS is re-signed on the way out -- a VPN, a managed network, antivirus that
//! inspects traffic -- the update check alone fails with `UnknownIssuer` while every other
//! program on it is fine, and no setting in WoodSketch could help (#14). So the requests are
//! made here instead.
//!
//! What is read is a feed per release, not the release list itself: `vpk` uploads a
//! `releases.<channel>.json` beside each platform's package, and each names the packages of that
//! release alone. They are merged newest-first across the last [`RELEASES`] releases, which is
//! what Velopack then picks a version out of, and what it checks a download against.

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc::Sender;

use serde::Deserialize;
use velopack::bundle::Manifest;
use velopack::sources::UpdateSource;
use velopack::{Error, VelopackAsset, VelopackAssetFeed};
use woodsketch_ui_egui::net;

/// How many releases back to look. Releases here are small and frequent, and a channel only needs
/// its newest package, so this is about tolerating a release that carries no package for *this*
/// platform rather than about reaching far back.
const RELEASES: usize = 10;

/// Read in 2 MB pieces, as Velopack does: a package is tens of megabytes.
const PIECE: usize = 2 * 1024 * 1024;

/// One release as the GitHub API gives it, with only the fields used here.
#[derive(Deserialize)]
struct Release {
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    assets: Vec<Upload>,
}

/// One file attached to a release.
#[derive(Deserialize)]
struct Upload {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    browser_download_url: Option<String>,
}

/// A GitHub repository's releases, as somewhere updates come from.
pub struct Releases {
    /// The API address listing them, newest first.
    listing: String,
}

impl Releases {
    /// `repo` is the repository's web address, as `https://github.com/FMastro/woodsketch`.
    pub fn at(repo: &str) -> Self {
        Self { listing: listing_url(repo) }
    }

    /// The published releases, newest first, drafts and pre-releases left out.
    fn published(&self) -> Result<Vec<Release>, Error> {
        let json = get(&self.listing, "application/vnd.github.v3+json")?;
        let mut releases: Vec<Release> = serde_json::from_str(&json)?;
        releases.retain(|r| !r.prerelease);
        releases.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        Ok(releases)
    }
}

/// The API address for a repository's releases, from its web address.
fn listing_url(repo: &str) -> String {
    let path = repo.trim_end_matches('/');
    let mut parts = path.rsplit('/');
    let name = parts.next().unwrap_or_default();
    let owner = parts.next().unwrap_or_default();
    format!("https://api.github.com/repos/{owner}/{name}/releases?per_page={RELEASES}&page=1")
}

/// Where one of a release's files can be downloaded from, by name.
fn upload_url<'a>(release: &'a Release, name: &str) -> Option<&'a str> {
    let named = release.assets.iter().find(|a| a.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case(name)))?;
    named.browser_download_url.as_deref()
}

/// Every HTTPS client in turn, until one isn't refused the certificate.
fn get(url: &str, accept: &str) -> Result<String, Error> {
    let mut refused = None;
    for roots in net::roots() {
        let agent: ureq::Agent =
            ureq::Agent::config_builder().tls_config(net::tls(roots.clone())).timeout_global(Some(std::time::Duration::from_secs(30))).build().into();
        match agent.get(url).header("Accept", accept).call() {
            Ok(mut response) => return Ok(response.body_mut().read_to_string()?),
            Err(e) if net::untrusted_certificate(&e) => {
                log::warn!("{roots:?} wouldn't have it, trying the next roots: {e}");
                refused = Some(e);
            }
            Err(e) => return Err(e.into()),
        }
    }
    Err(refused.expect("a loop over the roots either answers or is refused one").into())
}

/// Downloads a file, saying how far along it is in steps of 5%, as Velopack's own download does
/// (a step per piece would be a notice every few hundredths of a second).
///
/// No overall timeout: a package is tens of megabytes and a slow line is not a failure. The
/// certificate is settled before the first byte arrives, so the fall-back onto the next roots
/// never means downloading anything twice.
fn download(url: &str, to: &Path, progress: Option<Sender<i16>>) -> Result<(), Error> {
    let mut refused = None;
    for roots in net::roots() {
        let agent: ureq::Agent = ureq::Agent::config_builder().tls_config(net::tls(roots.clone())).build().into();
        let (head, body) = match agent.get(url).header("Accept", "application/octet-stream").call() {
            Ok(response) => response.into_parts(),
            Err(e) if net::untrusted_certificate(&e) => {
                log::warn!("{roots:?} wouldn't have it, trying the next roots: {e}");
                refused = Some(e);
                continue;
            }
            Err(e) => return Err(e.into()),
        };
        let total = head.headers.get("Content-Length").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok());
        let mut file = File::create(to)?;
        let mut reader = body.into_reader();
        let mut piece = vec![0; PIECE];
        let (mut done, mut said) = (0u64, 0i16);
        loop {
            let read = reader.read(&mut piece)?;
            if read == 0 {
                break;
            }
            file.write_all(&piece[..read])?;
            done += read as u64;
            if let (Some(total), Some(progress)) = (total, progress.as_ref())
                && total > 0
            {
                let step = (done as f64 / total as f64 * 20.0).floor() as i16 * 5;
                if step > said {
                    said = step;
                    let _ = progress.send(step);
                }
            }
        }
        return Ok(());
    }
    Err(refused.expect("a loop over the roots either answers or is refused one").into())
}

impl UpdateSource for Releases {
    fn get_release_feed(&self, channel: &str, _app: &Manifest, _staged_user_id: &str) -> Result<VelopackAssetFeed, Error> {
        let wanted = format!("releases.{channel}.json");
        let mut assets: Vec<VelopackAsset> = Vec::new();
        let mut went_wrong = None;
        for release in self.published()? {
            // A release with no feed for this channel is one built before the channel existed, or
            // one whose build for this platform failed: the next release back still has it.
            let Some(url) = upload_url(&release, &wanted) else { continue };
            match get(url, "application/octet-stream").and_then(|json| Ok(serde_json::from_str::<VelopackAssetFeed>(&json)?)) {
                Ok(feed) => assets.extend(feed.Assets),
                Err(e) => {
                    log::warn!("couldn't read {wanted} from a release: {e}");
                    went_wrong = Some(e);
                }
            }
        }
        // Nothing read and something went wrong is a failure, not an up-to-date answer: silence
        // would leave the app sitting on an old version saying it was the newest.
        match went_wrong {
            Some(e) if assets.is_empty() => Err(e),
            _ => Ok(VelopackAssetFeed { Assets: assets }),
        }
    }

    fn download_release_entry(&self, asset: &VelopackAsset, local_file: &Path, progress_sender: Option<Sender<i16>>) -> Result<(), Error> {
        for release in self.published()? {
            if let Some(url) = upload_url(&release, &asset.FileName) {
                log::info!("downloading {} to {}", asset.FileName, local_file.display());
                return download(url, local_file, progress_sender);
            }
        }
        Err(Error::Other(format!("'{}' isn't attached to any of the last {RELEASES} releases.", asset.FileName)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_repository_address_becomes_its_release_listing() {
        assert_eq!(listing_url("https://github.com/FMastro/woodsketch"), "https://api.github.com/repos/FMastro/woodsketch/releases?per_page=10&page=1");
        assert_eq!(
            listing_url("https://github.com/FMastro/woodsketch/"),
            listing_url("https://github.com/FMastro/woodsketch"),
            "a trailing slash is the same repository"
        );
    }

    fn release(from: &str) -> Release {
        serde_json::from_str(from).expect("the test's own JSON")
    }

    /// The feed is found by name whatever case GitHub reports it in, and a release without it is
    /// passed over rather than failing the check.
    #[test]
    fn a_releases_files_are_found_by_name() {
        let r = release(
            r#"{"prerelease": false, "published_at": "2026-11-16T04:41:13Z", "assets": [
                {"name": "RELEASES.osx-arm64.JSON", "browser_download_url": "https://example.test/feed"},
                {"name": "WoodSketch-osx-arm64.nupkg", "browser_download_url": "https://example.test/pkg"}]}"#,
        );
        assert_eq!(upload_url(&r, "releases.osx-arm64.json"), Some("https://example.test/feed"));
        assert_eq!(upload_url(&r, "WoodSketch-osx-arm64.nupkg"), Some("https://example.test/pkg"));
        assert_eq!(upload_url(&r, "releases.win-x64.json"), None, "a channel this release has no package for");
    }

    /// Only the fields that matter are read, so a release carrying anything else still parses:
    /// the API answer is a few kilobytes of fields this doesn't know about.
    #[test]
    fn a_release_reads_despite_everything_else_github_says() {
        let r = release(r#"{"id": 1, "tag_name": "v0.1.74", "draft": false, "prerelease": true, "author": {"login": "someone"}}"#);
        assert!(r.prerelease);
        assert!(r.assets.is_empty());
        assert!(r.published_at.is_none(), "a release that was never published sorts last");
    }

    /// Every channel the release workflow builds, read out of the workflow itself: a platform
    /// added to that matrix and forgotten here would otherwise be a channel nothing ever checks.
    fn channels() -> Vec<String> {
        let yaml = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/release.yml")).expect("the release workflow");
        let channels: Vec<String> = yaml
            .lines()
            .filter_map(|l| l.trim().strip_prefix("- {"))
            .filter_map(|l| l.split("channel:").nth(1))
            .map(|rest| rest.split(',').next().unwrap_or_default().trim().to_owned())
            .collect();
        assert!(channels.contains(&"win-x64".to_owned()), "the matrix was read but Windows isn't in it: {channels:?}");
        channels
    }

    /// The channel names come out of the workflow, without the network: this is what tells whether
    /// [`channels`] still understands the matrix after someone reformats it.
    #[test]
    fn the_release_workflows_channels_can_be_read() {
        let channels = channels();
        assert!(channels.len() >= 4, "every platform has a channel: {channels:?}");
        assert!(channels.iter().all(|c| c.starts_with("win-") || c.starts_with("osx-") || c.starts_with("linux-")), "{channels:?}");
    }

    /// The real thing, against the real repository: every platform's channel has a package to
    /// update to, read over the real connection. This is the only test that can tell whether the
    /// feed GitHub actually serves still parses, and it is what stands behind every update anyone
    /// gets, so run it after touching this file: `WOODSKETCH_RELEASES=1 cargo test -p woodsketch`.
    /// Off by default because it needs the network and spends GitHub's unauthenticated rate limit.
    ///
    /// A channel whose first release hasn't been published yet has no feed anywhere, and that is
    /// said rather than failed: the Arm channels were in the matrix before any release carried
    /// them. A channel that *has* a feed must have a whole package in it.
    #[test]
    fn every_platform_has_an_update_to_find() {
        if std::env::var_os("WOODSKETCH_RELEASES").is_none() {
            return;
        }
        let releases = Releases::at(crate::update::REPO);
        // The computer's own roots are enough on their own: the shipped ones are a fall-back for a
        // machine with no trust store worth asking, not how this is meant to work. Without this,
        // the rest of the test would pass just as well on a silent fall-back every time.
        let [computer, _shipped] = net::roots();
        let own: ureq::Agent = ureq::Agent::config_builder().tls_config(net::tls(computer)).build().into();
        own.get(&releases.listing).header("Accept", "application/vnd.github.v3+json").call().expect("the roots this computer trusts reach GitHub");
        let mut live = 0;
        for channel in channels() {
            let feed = releases.get_release_feed(&channel, &Manifest::default(), "").unwrap_or_else(|e| panic!("{channel}: {e}"));
            if feed.Assets.is_empty() {
                println!("{channel}: no release has carried this channel yet");
                continue;
            }
            let newest = feed.Assets.iter().filter(|a| a.Type.eq_ignore_ascii_case("Full")).max_by(|a, b| a.Version.cmp(&b.Version));
            let newest = newest.unwrap_or_else(|| panic!("{channel} has no full package in {} assets", feed.Assets.len()));
            println!("{channel}: {} {} ({} bytes)", newest.FileName, newest.Version, newest.Size);
            assert!(newest.FileName.ends_with(".nupkg"), "{channel}: {}", newest.FileName);
            assert!(!newest.SHA256.is_empty(), "{channel}: nothing to check the download against");
            assert!(newest.Size > 0, "{channel}: an empty package");
            live += 1;
        }
        assert!(live >= 4, "only {live} channels had anything to update to, which can't be right");
    }
}
