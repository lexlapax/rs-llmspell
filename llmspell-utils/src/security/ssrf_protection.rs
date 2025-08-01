//! ABOUTME: SSRF (Server-Side Request Forgery) protection framework
//! ABOUTME: Validates URLs and prevents requests to internal networks and sensitive resources

use regex::Regex;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::sync::LazyLock;
use url::Url;

/// SSRF protection configuration
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct SsrfProtectionConfig {
    /// Block requests to private IP ranges
    pub block_private_ips: bool,
    /// Block requests to localhost
    pub block_localhost: bool,
    /// Block requests to link-local addresses
    pub block_link_local: bool,
    /// Block requests to reserved IP ranges
    pub block_reserved: bool,
    /// Allowed schemes (e.g., `["http", "https"]`)
    pub allowed_schemes: Vec<String>,
    /// Blocked schemes (e.g., `["file", "gopher", "dict"]`)
    pub blocked_schemes: Vec<String>,
    /// Allowed ports (empty = all ports allowed)
    pub allowed_ports: Vec<u16>,
    /// Blocked ports (e.g., [22, 23, 3389])
    pub blocked_ports: Vec<u16>,
    /// Whitelist of allowed hosts/domains
    pub allowed_hosts: Vec<String>,
    /// Blacklist of blocked hosts/domains
    pub blocked_hosts: Vec<String>,
    /// Maximum number of redirects to follow
    pub max_redirects: u32,
    /// Enable DNS rebinding protection
    pub dns_rebinding_protection: bool,
}

impl Default for SsrfProtectionConfig {
    fn default() -> Self {
        Self {
            block_private_ips: true,
            block_localhost: true,
            block_link_local: true,
            block_reserved: true,
            allowed_schemes: vec!["http".to_string(), "https".to_string()],
            blocked_schemes: vec![
                "file".to_string(),
                "gopher".to_string(),
                "dict".to_string(),
                "ftp".to_string(),
                "sftp".to_string(),
                "ldap".to_string(),
                "tftp".to_string(),
                "ssh".to_string(),
            ],
            allowed_ports: vec![], // Empty = all ports allowed
            blocked_ports: vec![
                22,    // SSH
                23,    // Telnet
                25,    // SMTP
                110,   // POP3
                135,   // Windows RPC
                139,   // NetBIOS
                445,   // SMB
                1433,  // MSSQL
                3306,  // MySQL
                3389,  // RDP
                5432,  // PostgreSQL
                5900,  // VNC
                6379,  // Redis
                8020,  // Hadoop NameNode
                9200,  // Elasticsearch
                11211, // Memcached
                27017, // MongoDB
            ],
            allowed_hosts: vec![],
            blocked_hosts: vec!["metadata.google.internal".to_string()],
            max_redirects: 5,
            dns_rebinding_protection: true,
        }
    }
}

/// SSRF protection validator
pub struct SsrfProtector {
    config: SsrfProtectionConfig,
}

impl SsrfProtector {
    /// Create a new SSRF protector with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: SsrfProtectionConfig::default(),
        }
    }

    /// Create SSRF protector with custom configuration
    #[must_use]
    pub fn with_config(config: SsrfProtectionConfig) -> Self {
        Self { config }
    }

    /// Validate a URL for SSRF risks
    ///
    /// # Errors
    ///
    /// Returns `SsrfError` if:
    /// - URL format is invalid
    /// - URL scheme is blocked or not allowed
    /// - Host is blocked or not in allowed list
    /// - Port is blocked or not in allowed list
    /// - IP address is in private/reserved ranges
    /// - Suspicious patterns are detected
    pub fn validate_url(&self, url_str: &str) -> Result<ValidatedUrl, SsrfError> {
        // Parse the URL
        let url = Url::parse(url_str).map_err(|e| SsrfError::InvalidUrl {
            url: url_str.to_string(),
            reason: e.to_string(),
        })?;

        // Check scheme
        self.validate_scheme(&url)?;

        // Check host
        let host = url
            .host()
            .ok_or_else(|| SsrfError::InvalidUrl {
                url: url_str.to_string(),
                reason: "No host in URL".to_string(),
            })?
            .to_string();

        // Check if host is blocked
        self.validate_host(&host)?;

        // Check port
        self.validate_port(&url)?;

        // Check for IP address and validate if present
        if let Ok(ip) = IpAddr::from_str(&host) {
            self.validate_ip_address(&ip)?;
        } else {
            // Check for localhost by name
            if self.config.block_localhost {
                let lower_host = host.to_lowercase();
                if lower_host == "localhost" || lower_host.ends_with(".localhost") {
                    return Err(SsrfError::BlockedHost {
                        host: host.to_string(),
                    });
                }
            }

            // Try to resolve host to check if it's an IP
            // For URL host strings like "[::1]", we need to strip brackets
            let host_to_check = if host.starts_with('[') && host.ends_with(']') {
                &host[1..host.len() - 1]
            } else {
                &host
            };

            if let Ok(ip) = IpAddr::from_str(host_to_check) {
                self.validate_ip_address(&ip)?;
            }
        }

        // Additional checks for specific patterns
        Self::check_url_patterns(&url)?;

        Ok(ValidatedUrl {
            original: url_str.to_string(),
            parsed: url,
            host,
        })
    }

    /// Validate the URL scheme
    fn validate_scheme(&self, url: &Url) -> Result<(), SsrfError> {
        let scheme = url.scheme();

        // Check blocked schemes
        if self.config.blocked_schemes.contains(&scheme.to_string()) {
            return Err(SsrfError::BlockedScheme {
                scheme: scheme.to_string(),
            });
        }

        // Check allowed schemes if specified
        if !self.config.allowed_schemes.is_empty()
            && !self.config.allowed_schemes.contains(&scheme.to_string())
        {
            return Err(SsrfError::DisallowedScheme {
                scheme: scheme.to_string(),
                allowed: self.config.allowed_schemes.clone(),
            });
        }

        Ok(())
    }

    /// Validate the host
    fn validate_host(&self, host: &str) -> Result<(), SsrfError> {
        // Check allowed hosts first if whitelist is configured
        if !self.config.allowed_hosts.is_empty() {
            let allowed = self
                .config
                .allowed_hosts
                .iter()
                .any(|allowed| host == allowed || host.ends_with(&format!(".{allowed}")));
            if !allowed {
                return Err(SsrfError::DisallowedHost {
                    host: host.to_string(),
                    allowed: self.config.allowed_hosts.clone(),
                });
            }
        }

        // Check blocked hosts
        for blocked in &self.config.blocked_hosts {
            if host == blocked || host.ends_with(&format!(".{blocked}")) {
                return Err(SsrfError::BlockedHost {
                    host: host.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate the port
    fn validate_port(&self, url: &Url) -> Result<(), SsrfError> {
        if let Some(port) = url.port_or_known_default() {
            // Check blocked ports
            if self.config.blocked_ports.contains(&port) {
                return Err(SsrfError::BlockedPort { port });
            }

            // Check allowed ports if specified
            if !self.config.allowed_ports.is_empty() && !self.config.allowed_ports.contains(&port) {
                return Err(SsrfError::DisallowedPort {
                    port,
                    allowed: self.config.allowed_ports.clone(),
                });
            }
        }

        Ok(())
    }

    /// Validate IP address
    fn validate_ip_address(&self, ip: &IpAddr) -> Result<(), SsrfError> {
        match ip {
            IpAddr::V4(ipv4) => self.validate_ipv4(*ipv4),
            IpAddr::V6(ipv6) => self.validate_ipv6(ipv6),
        }
    }

    /// Validate IPv4 address
    fn validate_ipv4(&self, ip: Ipv4Addr) -> Result<(), SsrfError> {
        // Check localhost
        if self.config.block_localhost && ip.is_loopback() {
            return Err(SsrfError::BlockedIP {
                ip: IpAddr::V4(ip),
                reason: "Localhost/loopback address".to_string(),
            });
        }

        // Check private ranges
        if self.config.block_private_ips && ip.is_private() {
            return Err(SsrfError::BlockedIP {
                ip: IpAddr::V4(ip),
                reason: "Private IP range".to_string(),
            });
        }

        // Check link-local
        if self.config.block_link_local && ip.is_link_local() {
            return Err(SsrfError::BlockedIP {
                ip: IpAddr::V4(ip),
                reason: "Link-local address".to_string(),
            });
        }

        // Check additional reserved ranges
        if self.config.block_reserved {
            // Check for reserved ranges not covered by standard methods
            let octets = ip.octets();

            // 0.0.0.0/8 - Current network
            if octets[0] == 0 {
                return Err(SsrfError::BlockedIP {
                    ip: IpAddr::V4(ip),
                    reason: "Reserved: Current network (0.0.0.0/8)".to_string(),
                });
            }

            // 100.64.0.0/10 - Shared address space
            if octets[0] == 100 && octets[1] >= 64 && octets[1] <= 127 {
                return Err(SsrfError::BlockedIP {
                    ip: IpAddr::V4(ip),
                    reason: "Reserved: Shared address space (100.64.0.0/10)".to_string(),
                });
            }

            // 224.0.0.0/4 - Multicast
            if octets[0] >= 224 && octets[0] <= 239 {
                return Err(SsrfError::BlockedIP {
                    ip: IpAddr::V4(ip),
                    reason: "Reserved: Multicast (224.0.0.0/4)".to_string(),
                });
            }

            // 240.0.0.0/4 - Reserved for future use (excluding 255.255.255.255 broadcast)
            if octets[0] >= 240 && octets[0] < 255 {
                return Err(SsrfError::BlockedIP {
                    ip: IpAddr::V4(ip),
                    reason: "Reserved: Future use (240.0.0.0/4)".to_string(),
                });
            }

            // 255.255.255.255 - Broadcast
            if octets[0] == 255 && octets[1] == 255 && octets[2] == 255 && octets[3] == 255 {
                return Err(SsrfError::BlockedIP {
                    ip: IpAddr::V4(ip),
                    reason: "Reserved: Broadcast address".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate IPv6 address
    fn validate_ipv6(&self, ip: &Ipv6Addr) -> Result<(), SsrfError> {
        // Check localhost
        if self.config.block_localhost && ip.is_loopback() {
            return Err(SsrfError::BlockedIP {
                ip: IpAddr::V6(*ip),
                reason: "Localhost/loopback address".to_string(),
            });
        }

        // Check link-local
        if self.config.block_link_local {
            let segments = ip.segments();
            // fe80::/10
            if segments[0] & 0xffc0 == 0xfe80 {
                return Err(SsrfError::BlockedIP {
                    ip: IpAddr::V6(*ip),
                    reason: "Link-local address".to_string(),
                });
            }
        }

        // Check private ranges (Unique Local Addresses)
        if self.config.block_private_ips {
            let segments = ip.segments();
            // fc00::/7
            if segments[0] & 0xfe00 == 0xfc00 {
                return Err(SsrfError::BlockedIP {
                    ip: IpAddr::V6(*ip),
                    reason: "Private IPv6 range (Unique Local)".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Check for suspicious URL patterns
    fn check_url_patterns(url: &Url) -> Result<(), SsrfError> {
        // Detect attempts to bypass filters
        static BYPASS_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
            vec![
                // Decimal IP notation (but not regular dotted notation)
                Regex::new(r"^https?://\d{8,10}/?").unwrap(),
                // Octal IP notation
                Regex::new(r"^https?://0\d+\.").unwrap(),
                // Hex IP notation
                Regex::new(r"^https?://0x[0-9a-fA-F]+").unwrap(),
                // URL encoding bypass attempts
                Regex::new(r"%00|%0[aA]|%0[dD]").unwrap(),
                // Double URL encoding
                Regex::new(r"%25[0-9a-fA-F]{2}").unwrap(),
            ]
        });

        let url_str = url.as_str();
        for pattern in BYPASS_PATTERNS.iter() {
            if pattern.is_match(url_str) {
                return Err(SsrfError::SuspiciousPattern {
                    pattern: pattern.as_str().to_string(),
                });
            }
        }

        Ok(())
    }

    /// Check if a redirect URL is safe
    ///
    /// # Errors
    ///
    /// Returns `SsrfError` if:
    /// - Either URL is invalid
    /// - Target URL fails SSRF validation
    /// - Protocol downgrade is detected (HTTPS to HTTP)
    pub fn validate_redirect(&self, from_url: &str, to_url: &str) -> Result<(), SsrfError> {
        // Validate the target URL
        self.validate_url(to_url)?;

        // Additional checks for redirects
        let from = Url::parse(from_url).map_err(|_| SsrfError::InvalidUrl {
            url: from_url.to_string(),
            reason: "Invalid source URL".to_string(),
        })?;

        let to = Url::parse(to_url).map_err(|_| SsrfError::InvalidUrl {
            url: to_url.to_string(),
            reason: "Invalid target URL".to_string(),
        })?;

        // Check for protocol downgrade
        if from.scheme() == "https" && to.scheme() == "http" {
            return Err(SsrfError::ProtocolDowngrade {
                from: from.scheme().to_string(),
                to: to.scheme().to_string(),
            });
        }

        Ok(())
    }
}

impl Default for SsrfProtector {
    fn default() -> Self {
        Self::new()
    }
}

/// Validated URL that passed SSRF checks
#[derive(Debug, Clone)]
pub struct ValidatedUrl {
    /// Original URL string
    pub original: String,
    /// Parsed URL
    pub parsed: Url,
    /// Host component
    pub host: String,
}

/// SSRF validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum SsrfError {
    /// Invalid URL format
    InvalidUrl {
        /// The invalid URL
        url: String,
        /// Reason for invalidity
        reason: String,
    },
    /// Blocked URL scheme
    BlockedScheme {
        /// The blocked scheme
        scheme: String,
    },
    /// Disallowed URL scheme
    DisallowedScheme {
        /// The disallowed scheme
        scheme: String,
        /// List of allowed schemes
        allowed: Vec<String>,
    },
    /// Blocked host
    BlockedHost {
        /// The blocked host
        host: String,
    },
    /// Disallowed host
    DisallowedHost {
        /// The disallowed host
        host: String,
        /// List of allowed hosts
        allowed: Vec<String>,
    },
    /// Blocked port
    BlockedPort {
        /// The blocked port number
        port: u16,
    },
    /// Disallowed port
    DisallowedPort {
        /// The disallowed port number
        port: u16,
        /// List of allowed ports
        allowed: Vec<u16>,
    },
    /// Blocked IP address
    BlockedIP {
        /// The blocked IP address
        ip: IpAddr,
        /// Reason for blocking
        reason: String,
    },
    /// Suspicious pattern detected
    SuspiciousPattern {
        /// The detected pattern
        pattern: String,
    },
    /// Protocol downgrade detected
    ProtocolDowngrade {
        /// Original protocol
        from: String,
        /// Downgraded protocol
        to: String,
    },
}

impl std::fmt::Display for SsrfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl { url, reason } => write!(f, "Invalid URL '{url}': {reason}"),
            Self::BlockedScheme { scheme } => write!(f, "Blocked scheme: {scheme}"),
            Self::DisallowedScheme { scheme, allowed } => {
                write!(f, "Scheme '{scheme}' not allowed. Allowed: {allowed:?}")
            }
            Self::BlockedHost { host } => write!(f, "Blocked host: {host}"),
            Self::DisallowedHost { host, allowed } => {
                write!(f, "Host '{host}' not allowed. Allowed: {allowed:?}")
            }
            Self::BlockedPort { port } => write!(f, "Blocked port: {port}"),
            Self::DisallowedPort { port, allowed } => {
                write!(f, "Port {port} not allowed. Allowed: {allowed:?}")
            }
            Self::BlockedIP { ip, reason } => write!(f, "Blocked IP {ip}: {reason}"),
            Self::SuspiciousPattern { pattern } => {
                write!(f, "Suspicious pattern detected: {pattern}")
            }
            Self::ProtocolDowngrade { from, to } => {
                write!(f, "Protocol downgrade from {from} to {to}")
            }
        }
    }
}

impl std::error::Error for SsrfError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_block_private_ips() {
        let protector = SsrfProtector::new();

        // Should block private IPs
        assert!(protector.validate_url("http://192.168.1.1").is_err());
        assert!(protector.validate_url("http://10.0.0.1").is_err());
        assert!(protector.validate_url("http://172.16.0.1").is_err());

        // Should allow public IPs
        assert!(protector.validate_url("http://8.8.8.8").is_ok());
        assert!(protector.validate_url("https://example.com").is_ok());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_block_localhost() {
        let protector = SsrfProtector::new();

        // Should block localhost
        assert!(protector.validate_url("http://localhost").is_err());
        assert!(protector.validate_url("http://127.0.0.1").is_err());
        assert!(protector.validate_url("http://[::1]").is_err());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_blocked_schemes() {
        let protector = SsrfProtector::new();

        // Should block dangerous schemes
        assert!(protector.validate_url("file:///etc/passwd").is_err());
        assert!(protector.validate_url("gopher://example.com").is_err());
        assert!(protector.validate_url("dict://example.com").is_err());

        // Should allow safe schemes
        assert!(protector.validate_url("http://example.com").is_ok());
        assert!(protector.validate_url("https://example.com").is_ok());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_blocked_ports() {
        let protector = SsrfProtector::new();

        // Should block dangerous ports
        assert!(protector.validate_url("http://example.com:22").is_err());
        assert!(protector.validate_url("http://example.com:3306").is_err());
        assert!(protector.validate_url("http://example.com:6379").is_err());

        // Should allow safe ports
        assert!(protector.validate_url("http://example.com:80").is_ok());
        assert!(protector.validate_url("https://example.com:443").is_ok());
        assert!(protector.validate_url("http://example.com:8080").is_ok());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_suspicious_patterns() {
        let protector = SsrfProtector::new();

        // Should detect bypass attempts
        assert!(protector.validate_url("http://2130706433").is_err()); // Decimal IP
        assert!(protector.validate_url("http://0x7f000001").is_err()); // Hex IP
        assert!(protector
            .validate_url("http://example.com%00.evil.com")
            .is_err()); // Null byte
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_ipv6_blocking() {
        let protector = SsrfProtector::new();

        // Should block IPv6 localhost
        assert!(protector.validate_url("http://[::1]").is_err());

        // Should block link-local
        assert!(protector.validate_url("http://[fe80::1]").is_err());

        // Should block unique local addresses
        assert!(protector.validate_url("http://[fc00::1]").is_err());
        assert!(protector.validate_url("http://[fd00::1]").is_err());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_custom_config() {
        let config = SsrfProtectionConfig {
            allowed_hosts: vec!["trusted.com".to_string()],
            allowed_ports: vec![80, 443],
            ..Default::default()
        };

        let protector = SsrfProtector::with_config(config);

        // Should only allow whitelisted hosts
        assert!(protector.validate_url("https://trusted.com").is_ok());
        assert!(protector.validate_url("https://untrusted.com").is_err());

        // Should only allow whitelisted ports
        assert!(protector.validate_url("http://trusted.com:80").is_ok());
        assert!(protector.validate_url("https://trusted.com:443").is_ok());
        assert!(protector.validate_url("http://trusted.com:8080").is_err());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_redirect_validation() {
        let protector = SsrfProtector::new();

        // Should allow safe redirects
        assert!(protector
            .validate_redirect("https://example.com", "https://example.com/page")
            .is_ok());

        // Should block protocol downgrade
        assert!(protector
            .validate_redirect("https://example.com", "http://example.com")
            .is_err());

        // Should block redirect to private IP
        assert!(protector
            .validate_redirect("https://example.com", "http://192.168.1.1")
            .is_err());
    }
}
