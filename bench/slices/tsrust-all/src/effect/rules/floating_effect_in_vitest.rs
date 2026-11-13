//! Port of Effect-TS/tsgo `internal/rules/floating_effect_in_vitest.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;

// Go: rules/floating_effect_in_vitest.go vitestCallbackKind
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum VitestCallbackKind {
    None = 0,
    Test = 1,
    Hook = 2,
}

// Go: rules/floating_effect_in_vitest.go vitestRoot
#[derive(Clone, Copy, Default, Debug)]
struct VitestRoot {
    name: &'static str,
    effect_vitest: bool,
}

// Go: rules/floating_effect_in_vitest.go vitestTestModifiers
static VITEST_TEST_MODIFIERS: &[&str] = &[
    "concurrent",
    "each",
    "fails",
    "for",
    "only",
    "prop",
    "runIf",
    "sequential",
    "skip",
    "skipIf",
    "todo",
];

// Go: rules/floating_effect_in_vitest.go vitestHookNames
static VITEST_HOOK_NAMES: &[&str] = &["afterAll", "afterEach", "beforeAll", "beforeEach"];

// Go: rules/floating_effect_in_vitest.go effectAwareVitestMethods
static EFFECT_AWARE_VITEST_METHODS: &[&str] = &["effect", "live", "scoped", "scopedLive"];



// Go: rules/floating_effect_in_vitest.go floatingEffectVitestCallback

// Go: rules/floating_effect_in_vitest.go matchVitestCallee

// Go: rules/floating_effect_in_vitest.go matchVitestRoot

// Go: rules/floating_effect_in_vitest.go classifyVitestCallback
fn classify_vitest_callback(root: VitestRoot, members: &[String]) -> VitestCallbackKind {
    if root.name.is_empty() {
        return VitestCallbackKind::None;
    }
    if VITEST_HOOK_NAMES.contains(&root.name) {
        if members.is_empty() {
            return VitestCallbackKind::Hook;
        }
        return VitestCallbackKind::None;
    }

    if root.effect_vitest
        && !members.is_empty()
        && EFFECT_AWARE_VITEST_METHODS.contains(&members[0].as_str())
    {
        return VitestCallbackKind::None;
    }
    if members.len() == 1 && VITEST_HOOK_NAMES.contains(&members[0].as_str()) {
        return VitestCallbackKind::Hook;
    }
    for member in members {
        if !VITEST_TEST_MODIFIERS.contains(&member.as_str()) {
            return VitestCallbackKind::None;
        }
    }
    VitestCallbackKind::Test
}

// Go: rules/floating_effect_in_vitest.go vitestCallbackReturnsEffect

// Go: rules/floating_effect_in_vitest.go vitestReturnTypeContainsEffect
