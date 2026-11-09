//! Go `internal/project/extendedconfigcache.go`.

use crate::project::prelude::*;


// Go: project/extendedconfigcache.go:9 ExtendedConfigParseArgs
// PORT: the cache stores its parse function for any args, so the args own
// their values: Go interfaces are `Rc<dyn ..>` (a nil `Cache` is `None`).
// Go passes the config file registry builder as `Host` and `Cache`, so the
// builder hands out `Rc`s of itself.

// Go: project/extendedconfigcache.go:18 ExtendedConfigCacheEntry
// PORT: Go embeds `*tsoptions.ExtendedConfigCacheEntry`; here it is the
// field `extended_config_cache_entry` (map-project.md section 4). Go
// `xxh3.Uint128` is `u128`.

// Go: project/extendedconfigcache.go:23 ExtendedConfigCache

// Go: project/extendedconfigcache.go:25 NewExtendedConfigCache

// Go: project/extendedconfigcache.go:40 hash
// PORT: Go passes the args by value; here by reference.
