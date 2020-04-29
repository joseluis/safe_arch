#![no_std]
#![warn(missing_docs)]
#![allow(unused_imports)]

//! Crate that safely exposes arch intrinsics via cfg.
//!
//! >Incomplete. WIP. Etc.
//! >
//! >Current content can be expected to remain stable, but the functionality
//! >coverage is not that much.
//!
//! This crate lets you safely use CPU intrinsics. Those things in
//! [`core::arch`](core::arch).
//! * Most of them are 100% safe to use as long as the CPU feature is available,
//!   like addition and multiplication and stuff.
//! * Some of them require that you uphold extra alignment requirements or
//!   whatever, which we do via the type system when necessary.
//! * Some of them are absolutely not safe at all because it causes UB at the
//!   LLVM level, so those things are not exposed here.
//!
//! This crate works purely via `cfg` and compile time feature selection, there
//! are no runtime checks. This means that if you _do_ want to do runtime
//! feature detection and then dynamically call an intrinsic if it happens to be
//! available, then this crate sadly isn't for you.
//!
//! ## Compile Time CPU Target Features
//!
//! At the time of me writing this, Rust enables the `sse` and `sse2` CPU
//! features by default for all `i686` (x86) and `x86_64` builds. Those CPU
//! features are built into the design of `x86_64`, and you'd need a _super_ old
//! `x86` CPU for it to not support at least `sse` and `sse2`, so they're a safe
//! bet for the language to enable all the time. In fact, because the standard
//! library is compiled with them enabled, simply trying to _disable_ those
//! features would actually cause ABI issues and fill your program with UB
//! ([link][rustc_docs]).
//!
//! If you want additional CPU features available at compile time you'll have to
//! enable them with an additional arg to `rustc`. For a feature named `name`
//! you pass `-C target-feature=+name`, such as `-C target-feature=+sse3` for
//! `sse3`.
//!
//! You can alternately enable _all_ target features of the current CPU with `-C
//! target-cpu=native`. This is primarily of use if you're building a program
//! you'll only run on your own system.
//!
//! It's sometimes hard to know if your target platform will support a given
//! feature set, but the [Steam Hardware Survey][steam-survey] is generally
//! taken as a guide to what you can expect people to have available. If you
//! click "Other Settings" it'll expand into a list of CPU target features and
//! how common they are. These days, it seems that `sse3` can be safely assumed,
//! and `ssse3`, `sse4.1`, and `sse4.2` are pretty safe bets as well. The stuff
//! above 128-bit isn't as common yet, give it another few years.
//!
//! **Please note that executing a program on a CPU that doesn't support the
//! target features it was compiles for is Undefined Behavior.**
//!
//! Currently, Rust doesn't actually support an easy way for you to check that a
//! feature enabled at compile time is _actually_ available at runtime. There is
//! the "[feature_detected][feature_detected]" family of macros, but if you
//! enable a feature they will evaluate to a constant `true` instead of actually
//! deferring the check for the feature to runtime. This means that, if you
//! _did_ want a check at the start of your program, to confirm that all the
//! assumed features are present and error out when the assumptions don't hold,
//! you can't use that macro. You gotta use CPUID and check manually. rip.
//! Hopefully we can make that process easier in a future version of this crate.
//!
//! [steam-survey]:
//! https://store.steampowered.com/hwsurvey/Steam-Hardware-Software-Survey-Welcome-to-Steam
//! [feature_detected]:
//! https://doc.rust-lang.org/std/index.html?search=feature_detected
//! [rustc_docs]: https://doc.rust-lang.org/rustc/targets/known-issues.html
//!
//! ### A Note On Working With Cfg
//!
//! There's two main ways to use `cfg`:
//! * Via an attribute placed on an item, block, or expression:
//!   * `#[cfg(debug_assertions)] println!("hello");`
//! * Via a macro used within an expression position:
//!   * `if cfg!(debug_assertions) { println!("hello"); }`
//!
//! The difference might seem small but it's actually very important:
//! * The attribute form will include code or not _before_ deciding if all the
//!   items named and so forth really exist or not. This means that code that is
//!   configured via attribute can safely name things that don't always exist as
//!   long as the things they name do exist whenever that code is configured
//!   into the build.
//! * The macro form will include the configured code _no matter what_, and then
//!   the macro resolves to a constant `true` or `false` and the compiler uses
//!   dead code elimination to cut out the path not taken.
//!
//! This crate uses `cfg` via the attribute, so the functions it exposes don't
//! exist at all when the appropriate CPU target features aren't enabled.
//! Accordingly, if you plan to call this crate or not depending on what
//! features are enabled in the build you'll also need to control your use of
//! this crate via cfg attribute, not cfg macro.
//!
//! ## Current Support
//! As I said above, the crate is only Work In Progress status!
//!
//! * Intel (`x86` / `x86_64`)
//!   * `sse`

// https://en.wikipedia.org/wiki/CPUID#Calling_CPUID
// * first call __get_cpuid_max(0) and check ret.0 for the max leaf.
// * If a leaf has sub-leaves, call __get_cpuid_max(leaf) and check ret.1 for
//   that max.
// * once you know your limits, particular features can be checked for by
//   getting the info for a leaf and checking the bits of a particular return
//   register. Which bit you need to look for in what register in what leaf is
//   mostly covered in the wikipedia article, linked above.
// * Obviously we need to make checks for the most useful features available via
//   some helper functions in this crate.

use core::{
  convert::AsRef,
  fmt::{
    Binary, Debug, Display, LowerExp, LowerHex, Octal, UpperExp, UpperHex,
  },
  ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor,
    BitXorAssign, Div, DivAssign, Mul, MulAssign, Neg, Not, Sub, SubAssign,
  },
};

/// Declares a private mod and then a glob `use` with the visibility specified.
macro_rules! submodule {
  ($v:vis $name:ident) => {
    mod $name;
    $v use $name::*;
  };
  ($v:vis $name:ident { $($content:tt)* }) => {
    mod $name { $($content)* }
    $v use $name::*;
  };
}

// unlike with the `submodule!` macro, we _want_ to expose the existence these
// arch-specific modules.

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod intel {
  //! Types and functions for safe `x86` / `x86_64` intrinsic usage.
  //!
  //! `x86_64` is essentially a superset of `x86`, so we just lump it all into
  //! one module.
  use super::*;
  #[cfg(target_arch = "x86")]
  use core::arch::x86::*;
  #[cfg(target_arch = "x86_64")]
  use core::arch::x86_64::*;

  submodule!(pub m128_);
  submodule!(pub sse);
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use intel::*;