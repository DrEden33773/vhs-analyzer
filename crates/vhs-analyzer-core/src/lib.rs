#![warn(missing_docs)]

//! Core analysis library for VHS `.tape` files.
//!
//! Provides a resilient lexer and parser built on [`rowan`] lossless syntax
//! trees, plus formatting logic. This crate has no LSP or async dependencies
//! and can be embedded in any tooling context.

/// Shared syntax kinds and rowan language bindings.
pub mod syntax;

/// Hand-written lexical analysis for VHS source text.
pub mod lexer;
