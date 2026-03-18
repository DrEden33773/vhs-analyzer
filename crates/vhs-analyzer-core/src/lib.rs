#![warn(missing_docs)]

//! Core analysis library for VHS `.tape` files.
//!
//! Provides a resilient lexer and parser built on [`rowan`] lossless syntax
//! trees, plus formatting logic. This crate has no LSP or async dependencies
//! and can be embedded in any tooling context.
