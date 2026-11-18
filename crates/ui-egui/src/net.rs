//! The HTTPS WoodSketch speaks: checking for updates and sending a report. Which certificate
//! roots to trust is all there is to it, and it is one answer in one place because it decides
//! whether the app can reach anything at all.
//!
//! `ureq` ships Mozilla's root list compiled into the binary and trusts only that, deliberately:
//! a root the computer was told to trust could have been put there to read the traffic. But a
//! woodworking app runs on managed laptops and behind VPNs, and on such a machine every other
//! program works while WoodSketch alone can't reach GitHub, with `UnknownIssuer` and nothing to
//! do about it (#14). So the computer's own roots are asked first -- the same ones the browser
//! would use to download the release by hand -- and the roots shipped here are the fall-back,
//! for a machine with no trust store of its own worth asking.
//!
//! Only an untrusted certificate moves on to the next roots. Anything else is reported as it
//! happened, so a report is never posted twice because the first attempt failed *after* the relay
//! already had it.
//!
//! This decides who vouches for the connection, not whether what arrives is genuine: Velopack
//! checks a downloaded package against the SHA256 in the release feed before applying it, and the
//! Windows build is signed besides.

use ureq::tls::{RootCerts, TlsConfig};

/// The roots to trust, in the order to try them.
pub fn roots() -> [RootCerts; 2] {
    [RootCerts::PlatformVerifier, RootCerts::WebPki]
}

/// TLS settings trusting one of [`roots`].
pub fn tls(roots: RootCerts) -> TlsConfig {
    TlsConfig::builder().root_certs(roots).build()
}

/// Whether the request failed because the other end's certificate wasn't trusted, which is the
/// one failure worth trying the next roots for.
///
/// `ureq` has no variant of its own for it -- a `rustls` refusal arrives as an io error, so its
/// message is all there is to go on. Reading it wrong costs one wasted request, which is why this
/// decides a retry and never an error message.
pub fn untrusted_certificate(e: &ureq::Error) -> bool {
    matches!(e, ureq::Error::Io(_)) && e.to_string().contains("certificate")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The computer is asked first: on a network that re-signs HTTPS, its roots are the only ones
    /// that can work, and on an ordinary machine they hold the same public roots anyway.
    #[test]
    fn the_computers_own_roots_come_first() {
        assert!(matches!(roots(), [RootCerts::PlatformVerifier, RootCerts::WebPki]));
    }

    /// The message from the report that started this (#14) has to be recognized, since that is the
    /// failure the fall-back exists for.
    #[test]
    fn an_untrusted_certificate_is_told_from_an_unreachable_server() {
        let refused = ureq::Error::Io(std::io::Error::other("invalid peer certificate: UnknownIssuer"));
        assert!(untrusted_certificate(&refused));
        let unreachable = ureq::Error::Io(std::io::Error::from(std::io::ErrorKind::ConnectionRefused));
        assert!(!untrusted_certificate(&unreachable), "a refused connection is the network, not the certificate");
        assert!(!untrusted_certificate(&ureq::Error::HostNotFound), "nor is a name that doesn't resolve");
    }
}
