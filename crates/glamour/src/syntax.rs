//! Syntax highlighting support for code blocks.
//!
//! This module provides language detection and syntax highlighting
//! using the [syntect](https://crates.io/crates/syntect) library.
//!
//! # Example
//!
//! ```rust,ignore
//! use glamour::syntax::LanguageDetector;
//!
//! let detector = LanguageDetector::new();
//! let syntax = detector.detect("rust");
//! assert!(detector.is_supported("rust"));
//! assert!(detector.is_supported("rs")); // Alias works too
//! ```

use std::sync::LazyLock;
use syntect::parsing::{SyntaxReference, SyntaxSet};

/// Lazily loaded syntax set containing all default language definitions.
///
/// This is loaded on first use to avoid startup overhead when syntax
/// highlighting is not used.
pub static SYNTAX_SET: LazyLock<SyntaxSet> =
    LazyLock::new(|| SyntaxSet::load_defaults_newlines());

/// Maps markdown language identifiers to syntect syntax definitions.
///
/// Handles common language aliases (e.g., "js" → "javascript", "rs" → "rust")
/// and provides fallback to plain text for unknown languages.
#[derive(Debug, Clone)]
pub struct LanguageDetector;

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageDetector {
    /// Creates a new language detector.
    ///
    /// The underlying syntax set is lazily loaded on first use.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Finds the syntax definition for a language identifier.
    ///
    /// This method handles common aliases and performs case-insensitive matching.
    /// If the language is not recognized, returns the plain text syntax.
    ///
    /// # Arguments
    ///
    /// * `lang` - Language identifier from markdown code fence (e.g., "rust", "js", "py")
    ///
    /// # Returns
    ///
    /// A reference to the syntax definition. Never panics.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let detector = LanguageDetector::new();
    /// let syntax = detector.detect("rs"); // Returns Rust syntax
    /// assert_eq!(syntax.name, "Rust");
    /// ```
    #[must_use]
    pub fn detect(&self, lang: &str) -> &'static SyntaxReference {
        let lang_lower = lang.to_lowercase().trim().to_string();

        // Handle empty language string
        if lang_lower.is_empty() {
            return SYNTAX_SET.find_syntax_plain_text();
        }

        // Try direct match first (syntect's find_syntax_by_token is case-insensitive)
        if let Some(syntax) = SYNTAX_SET.find_syntax_by_token(&lang_lower) {
            return syntax;
        }

        // Try common aliases
        let canonical = Self::resolve_alias(&lang_lower);

        if canonical != lang_lower {
            if let Some(syntax) = SYNTAX_SET.find_syntax_by_token(canonical) {
                return syntax;
            }
        }

        // Try by file extension
        if let Some(syntax) = SYNTAX_SET.find_syntax_by_extension(&lang_lower) {
            return syntax;
        }

        // Fallback to plain text
        SYNTAX_SET.find_syntax_plain_text()
    }

    /// Resolves common language aliases to their canonical names.
    ///
    /// # Arguments
    ///
    /// * `lang` - Lowercase language identifier
    ///
    /// # Returns
    ///
    /// The canonical language name if an alias is found, otherwise the original.
    fn resolve_alias(lang: &str) -> &str {
        match lang {
            // JavaScript/TypeScript
            "js" | "mjs" | "cjs" => "javascript",
            "ts" | "mts" | "cts" => "typescript",
            "jsx" => "javascript",
            "tsx" => "typescript",

            // Rust
            "rs" => "rust",

            // Python
            "py" | "python3" | "py3" => "python",
            "pyw" => "python",

            // Ruby
            "rb" => "ruby",

            // Shell
            "sh" | "bash" | "zsh" | "fish" | "ksh" => "shell",
            "shell" => "bash",
            "shellscript" => "bash",

            // Markup
            "md" | "markdown" => "markdown",
            "htm" => "html",

            // Config files
            "yml" => "yaml",
            "dockerfile" => "docker",

            // C family
            "c++" | "cxx" | "hpp" | "hxx" | "cc" | "hh" => "cpp",
            "h" => "c",
            "objc" => "objective-c",
            "objcpp" | "objc++" => "objective-c++",

            // JVM languages
            "kt" | "kts" => "kotlin",
            "scala" => "scala",
            "groovy" => "groovy",
            "clj" | "cljs" | "cljc" => "clojure",

            // .NET languages
            "cs" | "csharp" => "c#",
            "fs" | "fsharp" => "f#",
            "vb" => "visual basic",

            // Go
            "go" | "golang" => "go",

            // Erlang/Elixir
            "ex" | "exs" => "elixir",
            "erl" | "hrl" => "erlang",

            // Haskell
            "hs" | "lhs" => "haskell",

            // Lisp family
            "el" | "elisp" | "emacs-lisp" => "lisp",
            "rkt" | "scm" | "ss" => "scheme",

            // ML family
            "ml" | "mli" => "ocaml",
            "sml" => "standard ml",

            // Data formats
            "jsonc" => "json",
            "json5" => "json",

            // Misc
            "tf" | "hcl" => "terraform",
            "tex" | "latex" => "latex",
            "r" => "r",
            "pl" | "pm" => "perl",
            "php" | "php3" | "php4" | "php5" | "php7" | "php8" | "phtml" => "php",
            "lua" => "lua",
            "swift" => "swift",
            "dart" => "dart",
            "vim" | "vimscript" => "viml",
            "ps1" | "psm1" | "psd1" => "powershell",
            "bat" | "cmd" => "batch file",
            "asm" | "s" | "S" => "assembly",
            "nim" => "nim",
            "zig" => "zig",
            "v" => "v",
            "crystal" | "cr" => "crystal",
            "d" => "d",
            "ada" | "adb" | "ads" => "ada",
            "fortran" | "f" | "f90" | "f95" | "f03" | "f08" => "fortran",
            "cobol" | "cob" | "cbl" => "cobol",
            "pascal" | "pas" => "pascal",
            "makefile" | "make" | "mk" => "makefile",
            "cmake" => "cmake",
            "nginx" => "nginx",
            "apache" => "apacheconf",
            "diff" | "patch" => "diff",
            "graphql" | "gql" => "graphql",
            "proto" | "protobuf" => "protocol buffers",
            "thrift" => "thrift",
            "svg" => "xml",
            "xslt" | "xsl" => "xml",
            "vue" => "vue",
            "svelte" => "svelte",
            "scss" => "scss",
            "sass" => "sass",
            "less" => "less",
            "styl" | "stylus" => "stylus",
            "pug" | "jade" => "pug",
            "haml" => "haml",
            "slim" => "slim",
            "erb" => "html (rails)",
            "ejs" => "ejs",
            "jinja" | "jinja2" | "j2" => "jinja2",
            "handlebars" | "hbs" => "handlebars",
            "mustache" => "mustache",
            "twig" => "twig",
            "nunjucks" | "njk" => "nunjucks",
            "liquid" => "liquid",

            // No alias found
            _ => lang,
        }
    }

    /// Checks if a language identifier is supported for syntax highlighting.
    ///
    /// A language is considered supported if it resolves to something other
    /// than plain text.
    ///
    /// # Arguments
    ///
    /// * `lang` - Language identifier to check
    ///
    /// # Returns
    ///
    /// `true` if the language has syntax highlighting support, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let detector = LanguageDetector::new();
    /// assert!(detector.is_supported("rust"));
    /// assert!(detector.is_supported("rs"));
    /// assert!(!detector.is_supported("unknown-lang"));
    /// ```
    #[must_use]
    pub fn is_supported(&self, lang: &str) -> bool {
        let syntax = self.detect(lang);
        syntax.name != "Plain Text"
    }

    /// Returns the number of supported language syntaxes.
    ///
    /// This counts the total number of syntax definitions available,
    /// not including aliases.
    #[must_use]
    pub fn syntax_count() -> usize {
        SYNTAX_SET.syntaxes().len()
    }

    /// Returns a list of all supported language identifiers.
    ///
    /// This includes both the canonical names from syntect and
    /// common aliases.
    #[must_use]
    pub fn supported_languages() -> Vec<&'static str> {
        vec![
            // Rust
            "rust",
            "rs",
            // Python
            "python",
            "py",
            "py3",
            // JavaScript/TypeScript
            "javascript",
            "js",
            "mjs",
            "cjs",
            "typescript",
            "ts",
            "mts",
            "jsx",
            "tsx",
            // Go
            "go",
            "golang",
            // C family
            "c",
            "cpp",
            "c++",
            "cxx",
            "h",
            "hpp",
            // Java/JVM
            "java",
            "kotlin",
            "kt",
            "scala",
            "groovy",
            "clojure",
            "clj",
            // .NET
            "csharp",
            "cs",
            "c#",
            "fsharp",
            "fs",
            "f#",
            // Ruby
            "ruby",
            "rb",
            // Shell
            "bash",
            "sh",
            "zsh",
            "shell",
            "fish",
            // Web
            "html",
            "htm",
            "css",
            "scss",
            "sass",
            "less",
            // Data formats
            "json",
            "jsonc",
            "yaml",
            "yml",
            "toml",
            "xml",
            "csv",
            // Markdown
            "markdown",
            "md",
            // SQL
            "sql",
            // Other
            "php",
            "perl",
            "pl",
            "lua",
            "swift",
            "objective-c",
            "objc",
            "r",
            "haskell",
            "hs",
            "elixir",
            "ex",
            "erlang",
            "erl",
            "ocaml",
            "ml",
            "lisp",
            "scheme",
            "makefile",
            "make",
            "dockerfile",
            "docker",
            "nginx",
            "diff",
            "patch",
            "graphql",
            "gql",
            "protobuf",
            "proto",
            "terraform",
            "tf",
            "hcl",
            "powershell",
            "ps1",
            "batch",
            "bat",
            "cmd",
            "vim",
            "viml",
            "latex",
            "tex",
            "asm",
            "assembly",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = LanguageDetector::new();
        // Should not panic
        let _ = detector.detect("rust");
    }

    #[test]
    fn test_direct_language_match() {
        let detector = LanguageDetector::new();

        // These should match directly
        assert!(detector.is_supported("rust"));
        assert!(detector.is_supported("python"));
        assert!(detector.is_supported("javascript"));
        assert!(detector.is_supported("html"));
        assert!(detector.is_supported("css"));
        assert!(detector.is_supported("json"));
        assert!(detector.is_supported("yaml"));
    }

    #[test]
    fn test_rust_aliases() {
        let detector = LanguageDetector::new();
        let rust = detector.detect("rust");

        assert_eq!(detector.detect("rs").name, rust.name);
        assert_eq!(detector.detect("RS").name, rust.name);
        assert_eq!(detector.detect("Rust").name, rust.name);
    }

    #[test]
    fn test_javascript_aliases() {
        let detector = LanguageDetector::new();
        let js = detector.detect("javascript");

        assert_eq!(detector.detect("js").name, js.name);
        assert_eq!(detector.detect("JS").name, js.name);
        assert_eq!(detector.detect("mjs").name, js.name);
        assert_eq!(detector.detect("cjs").name, js.name);
    }

    #[test]
    fn test_typescript_aliases() {
        let detector = LanguageDetector::new();
        let ts = detector.detect("typescript");

        assert_eq!(detector.detect("ts").name, ts.name);
        assert_eq!(detector.detect("TS").name, ts.name);
        assert_eq!(detector.detect("mts").name, ts.name);
        assert_eq!(detector.detect("cts").name, ts.name);
    }

    #[test]
    fn test_python_aliases() {
        let detector = LanguageDetector::new();
        let py = detector.detect("python");

        assert_eq!(detector.detect("py").name, py.name);
        assert_eq!(detector.detect("PY").name, py.name);
        assert_eq!(detector.detect("python3").name, py.name);
        assert_eq!(detector.detect("py3").name, py.name);
    }

    #[test]
    fn test_ruby_aliases() {
        let detector = LanguageDetector::new();
        let rb = detector.detect("ruby");

        assert_eq!(detector.detect("rb").name, rb.name);
        assert_eq!(detector.detect("RB").name, rb.name);
    }

    #[test]
    fn test_shell_aliases() {
        let detector = LanguageDetector::new();

        // These should all resolve to a shell-like syntax
        assert!(detector.is_supported("bash"));
        assert!(detector.is_supported("sh"));
        assert!(detector.is_supported("zsh"));
        assert!(detector.is_supported("shell"));
    }

    #[test]
    fn test_go_aliases() {
        let detector = LanguageDetector::new();
        let go = detector.detect("go");

        assert_eq!(detector.detect("golang").name, go.name);
        assert_eq!(detector.detect("Go").name, go.name);
    }

    #[test]
    fn test_cpp_aliases() {
        let detector = LanguageDetector::new();
        let cpp = detector.detect("cpp");

        assert_eq!(detector.detect("c++").name, cpp.name);
        assert_eq!(detector.detect("cxx").name, cpp.name);
        assert_eq!(detector.detect("CPP").name, cpp.name);
    }

    #[test]
    fn test_yaml_aliases() {
        let detector = LanguageDetector::new();
        let yaml = detector.detect("yaml");

        assert_eq!(detector.detect("yml").name, yaml.name);
        assert_eq!(detector.detect("YML").name, yaml.name);
    }

    #[test]
    fn test_markdown_aliases() {
        let detector = LanguageDetector::new();
        let md = detector.detect("markdown");

        assert_eq!(detector.detect("md").name, md.name);
        assert_eq!(detector.detect("MD").name, md.name);
    }

    #[test]
    fn test_case_insensitive() {
        let detector = LanguageDetector::new();

        // All of these should resolve to the same syntax
        let lower = detector.detect("rust");
        let upper = detector.detect("RUST");
        let mixed = detector.detect("Rust");

        assert_eq!(lower.name, upper.name);
        assert_eq!(lower.name, mixed.name);
    }

    #[test]
    fn test_unknown_language_fallback() {
        let detector = LanguageDetector::new();

        // Unknown languages should fall back to plain text
        let plain = detector.detect("totally-unknown-language-xyz123");
        assert_eq!(plain.name, "Plain Text");

        // is_supported should return false
        assert!(!detector.is_supported("totally-unknown-language-xyz123"));
    }

    #[test]
    fn test_empty_language() {
        let detector = LanguageDetector::new();

        let plain = detector.detect("");
        assert_eq!(plain.name, "Plain Text");

        assert!(!detector.is_supported(""));
    }

    #[test]
    fn test_whitespace_handling() {
        let detector = LanguageDetector::new();

        // Whitespace should be trimmed
        let rust = detector.detect("rust");
        assert_eq!(detector.detect("  rust  ").name, rust.name);
        assert_eq!(detector.detect("\trust\n").name, rust.name);
    }

    #[test]
    fn test_no_panic_on_any_input() {
        let detector = LanguageDetector::new();

        // Test various edge cases that might cause panics
        let _ = detector.detect("");
        let _ = detector.detect("   ");
        let _ = detector.detect("\n\n\n");
        let _ = detector.detect("a".repeat(1000).as_str());
        let _ = detector.detect("!@#$%^&*()");
        let _ = detector.detect("🦀");
        let _ = detector.detect("日本語");
        let _ = detector.detect("null");
        let _ = detector.detect("undefined");
        let _ = detector.detect("NaN");

        // None of these should panic
    }

    #[test]
    fn test_syntax_count() {
        let count = LanguageDetector::syntax_count();
        // syntect includes ~60 languages by default
        assert!(count >= 50, "Expected at least 50 syntaxes, got {}", count);
    }

    #[test]
    fn test_supported_languages_list() {
        let langs = LanguageDetector::supported_languages();

        // Should have at least 30 entries
        assert!(
            langs.len() >= 30,
            "Expected at least 30 languages, got {}",
            langs.len()
        );

        // Check some expected languages are in the list
        assert!(langs.contains(&"rust"));
        assert!(langs.contains(&"rs"));
        assert!(langs.contains(&"python"));
        assert!(langs.contains(&"py"));
        assert!(langs.contains(&"javascript"));
        assert!(langs.contains(&"js"));
    }

    #[test]
    fn test_csharp_aliases() {
        let detector = LanguageDetector::new();

        assert!(detector.is_supported("c#"));
        assert!(detector.is_supported("cs"));
        assert!(detector.is_supported("csharp"));
    }

    #[test]
    fn test_kotlin_aliases() {
        let detector = LanguageDetector::new();
        let kotlin = detector.detect("kotlin");

        assert_eq!(detector.detect("kt").name, kotlin.name);
        assert_eq!(detector.detect("kts").name, kotlin.name);
    }

    #[test]
    fn test_html_aliases() {
        let detector = LanguageDetector::new();
        let html = detector.detect("html");

        assert_eq!(detector.detect("htm").name, html.name);
    }

    #[test]
    fn test_docker_aliases() {
        let detector = LanguageDetector::new();

        // dockerfile should be recognized
        assert!(detector.is_supported("dockerfile"));
        assert!(detector.is_supported("docker"));
    }

    #[test]
    fn test_json_aliases() {
        let detector = LanguageDetector::new();
        let json = detector.detect("json");

        assert_eq!(detector.detect("jsonc").name, json.name);
        assert_eq!(detector.detect("json5").name, json.name);
    }

    #[test]
    fn test_default_impl() {
        let detector = LanguageDetector::default();
        assert!(detector.is_supported("rust"));
    }

    #[test]
    fn test_elixir_aliases() {
        let detector = LanguageDetector::new();
        let elixir = detector.detect("elixir");

        assert_eq!(detector.detect("ex").name, elixir.name);
        assert_eq!(detector.detect("exs").name, elixir.name);
    }

    #[test]
    fn test_haskell_aliases() {
        let detector = LanguageDetector::new();
        let haskell = detector.detect("haskell");

        assert_eq!(detector.detect("hs").name, haskell.name);
    }

    #[test]
    fn test_ocaml_aliases() {
        let detector = LanguageDetector::new();
        let ocaml = detector.detect("ocaml");

        assert_eq!(detector.detect("ml").name, ocaml.name);
        assert_eq!(detector.detect("mli").name, ocaml.name);
    }

    #[test]
    fn test_perl_aliases() {
        let detector = LanguageDetector::new();
        let perl = detector.detect("perl");

        assert_eq!(detector.detect("pl").name, perl.name);
        assert_eq!(detector.detect("pm").name, perl.name);
    }

    #[test]
    fn test_php_aliases() {
        let detector = LanguageDetector::new();
        let php = detector.detect("php");

        assert_eq!(detector.detect("php3").name, php.name);
        assert_eq!(detector.detect("php7").name, php.name);
        assert_eq!(detector.detect("phtml").name, php.name);
    }

    #[test]
    fn test_powershell_aliases() {
        let detector = LanguageDetector::new();
        let ps = detector.detect("powershell");

        assert_eq!(detector.detect("ps1").name, ps.name);
        assert_eq!(detector.detect("psm1").name, ps.name);
    }

    #[test]
    fn test_terraform_aliases() {
        let detector = LanguageDetector::new();

        assert!(detector.is_supported("terraform"));
        assert!(detector.is_supported("tf"));
        assert!(detector.is_supported("hcl"));
    }

    #[test]
    fn test_latex_aliases() {
        let detector = LanguageDetector::new();
        let latex = detector.detect("latex");

        assert_eq!(detector.detect("tex").name, latex.name);
    }

    #[test]
    fn test_makefile_aliases() {
        let detector = LanguageDetector::new();
        let makefile = detector.detect("makefile");

        assert_eq!(detector.detect("make").name, makefile.name);
        assert_eq!(detector.detect("mk").name, makefile.name);
    }

    #[test]
    fn test_diff_aliases() {
        let detector = LanguageDetector::new();
        let diff = detector.detect("diff");

        assert_eq!(detector.detect("patch").name, diff.name);
    }

    #[test]
    fn test_clojure_aliases() {
        let detector = LanguageDetector::new();
        let clj = detector.detect("clojure");

        assert_eq!(detector.detect("clj").name, clj.name);
        assert_eq!(detector.detect("cljs").name, clj.name);
        assert_eq!(detector.detect("cljc").name, clj.name);
    }

    #[test]
    fn test_erlang_aliases() {
        let detector = LanguageDetector::new();
        let erl = detector.detect("erlang");

        assert_eq!(detector.detect("erl").name, erl.name);
        assert_eq!(detector.detect("hrl").name, erl.name);
    }
}
