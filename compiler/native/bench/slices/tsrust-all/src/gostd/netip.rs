//! Go `net/netip` subset (go1.27.1 `src/net/netip/netip.go`,
//! `src/net/netip/uint128.go`; the ported functions are the same as in
//! go1.26.8): `ParseAddr` and
//! `Addr.Is4`, which `net/url` `parseHost` calls, and what they call. Other
//! `net/netip` functions are not ported.
//!
//! PORT: Go reads the bytes of a string. The port reads the Go bytes of the
//! port form (`scanner_util::go_string_bytes`), so indexes and slices are Go
//! byte offsets. A Go string made from a slice of those bytes is in the
//! value form (`scanner_util::go_value_from_bytes`).

use crate::prelude::*;

use crate::gostd::errors::{self, GoError};
use crate::gostd::strconv;
use std::fmt;

// Go: net/netip/uint128.go:13 uint128
/// uint128 represents a uint128 using two uint64s.
///
/// When the methods below mention a bit number, bit 0 is the most
/// significant bit (in hi) and bit 127 is the lowest (lo&1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Uint128 {
    hi: u64,
    lo: u64,
}

// Go: net/netip/netip.go:37 Addr
/// Addr represents an IPv4 or IPv6 address (with or without a scoped
/// addressing zone), similar to [net.IP] or [net.IPAddr].
///
/// Unlike [net.IP] or [net.IPAddr], Addr is a comparable value
/// type (it supports == and can be a map key) and is immutable.
///
/// The zero Addr is not a valid IP address.
/// Addr{} is distinct from both 0.0.0.0 and ::.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Addr {
    /// addr is the hi and lo bits of an IPv6 address. If z==z4,
    /// hi and lo contain the IPv4-mapped IPv6 address.
    ///
    /// hi and lo are constructed by interpreting a 16-byte IPv6
    /// address as a big-endian 128-bit number. The most significant
    /// bits of that number go into hi, the rest into lo.
    addr: Uint128,

    /// Details about the address, wrapped up together and canonicalized.
    ///
    /// PORT: Go `unique.Handle[addrDetail]`. Two handles are equal when
    /// their values are equal, so the port keeps the value. `None` is the
    /// zero handle (`z0`).
    z: Option<AddrDetail>,
}

// Go: net/netip/netip.go:60 addrDetail
/// addrDetail represents the details of an Addr, like address family and IPv6 zone.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct AddrDetail {
    is_v6: bool,     // IPv4 is false, IPv6 is true.
    zone_v6: String, // != "" only if IsV6 is true.
}

// Go: net/netip/netip.go:67 z0, z4, z6noz
// z0, z4, and z6noz are sentinel Addr.z values.
// See the Addr type's field docs.
const Z0: Option<AddrDetail> = None;
const Z4: Option<AddrDetail> = Some(AddrDetail {
    is_v6: false,
    zone_v6: String::new(),
});
const Z6NOZ: Option<AddrDetail> = Some(AddrDetail {
    is_v6: true,
    zone_v6: String::new(),
});

// Go: net/netip/netip.go:85 IPv6Unspecified
/// IPv6Unspecified returns the IPv6 unspecified address "::".
pub fn ipv6_unspecified() -> Addr {
    Addr {
        addr: Uint128::default(),
        z: Z6NOZ,
    }
}

// Go: net/netip/netip.go:91 AddrFrom4
/// AddrFrom4 returns the address of the IPv4 address given by the bytes in addr.
pub fn addr_from4(addr: [u8; 4]) -> Addr {
    Addr {
        addr: Uint128 {
            hi: 0,
            lo: 0xffff00000000
                | u64::from(addr[0]) << 24
                | u64::from(addr[1]) << 16
                | u64::from(addr[2]) << 8
                | u64::from(addr[3]),
        },
        z: Z4,
    }
}

// Go: net/netip/netip.go:101 AddrFrom16

// Go: net/netip/netip.go:114 ParseAddr

// Go: net/netip/netip.go:140 parseAddrError
#[derive(Clone, Debug, PartialEq, Eq)]
struct ParseAddrError {
    in_: String,       // the string given to ParseAddr
    msg: &'static str, // an explanation of the parse failure
    at: String,        // optionally, the unparsed portion of in at which the error occurred.
}

impl ParseAddrError {
}


// Go: net/netip/netip.go:154 parseIPv4Fields

// Go: net/netip/netip.go:195 parseIPv4

// Go: net/netip/netip.go:205 parseIPv6

impl Addr {
    // Go: net/netip/netip.go:460 Is4
    /// Is4 reports whether ip is an IPv4 address.
    ///
    /// It returns false for IPv4-mapped IPv6 addresses. See [Addr.Unmap].
    pub fn is4(&self) -> bool {
        self.z == Z4
    }

    // Go: net/netip/netip.go:473 Is6
    /// Is6 reports whether ip is an IPv6 address, including IPv4-mapped
    /// IPv6 addresses.
    pub fn is6(&self) -> bool {
        self.z != Z0 && self.z != Z4
    }

    // Go: net/netip/netip.go:491 WithZone
    /// WithZone returns an IP that's the same as ip but with the provided
    /// zone. If zone is empty, the zone is removed. If ip is an IPv4
    /// address, WithZone is a no-op and returns ip unchanged.
    pub fn with_zone(mut self, zone: &str) -> Addr {
        if !self.is6() {
            return self;
        }
        if zone.is_empty() {
            self.z = Z6NOZ;
            return self;
        }
        self.z = Some(AddrDetail {
            is_v6: true,
            zone_v6: zone.to_string(),
        });
        self
    }
}
