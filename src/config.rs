//! Configuration options and builder for the Bamboo input method engine.
//!
//! Allows fine-tuning tone placement rules, free tone marking flexibility,
//! and orthographic syllable validation.

/// Configuration options for the Bamboo engine.
///
/// Use [`Config::default()`] for the standard modern Vietnamese input setup,
/// or [`Config::builder()`] / [`ConfigBuilder`] to customize individual flags.
///
/// # Example
/// ```rust
/// use bamboo_core::Config;
///
/// let config = Config::builder()
///     .free_tone_marking(true)
///     .std_tone_style(true)
///     .auto_correct(false)
///     .build();
///
/// assert!(config.free_tone_marking);
/// assert!(config.std_tone_style);
/// assert!(!config.auto_correct);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// If `true`, allows typing tone marks at any position in the word (Free Tone Marking).
    /// For example, `hoangf` -> `hoàng`.
    ///
    /// Default: `true`.
    pub free_tone_marking: bool,
    /// If `true`, uses the standard (new) tone placement (e.g., `hòa`, `khỏe`).
    /// If `false`, uses the traditional (old) style (e.g., `hoà`, `khoẻ`).
    ///
    /// Default: `true`.
    pub std_tone_style: bool,
    /// If `true`, enables automatic spelling correction to ensure valid Vietnamese syllables.
    /// Invalid syllables (e.g. non-Vietnamese consonant clusters with marks) will automatically
    /// fall back to raw characters.
    ///
    /// Default: `true`.
    pub auto_correct: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { free_tone_marking: true, std_tone_style: true, auto_correct: true }
    }
}

impl Config {
    /// Creates a new configuration with all standard defaults enabled.
    pub const fn new() -> Self {
        Self { free_tone_marking: true, std_tone_style: true, auto_correct: true }
    }

    /// Returns a [`ConfigBuilder`] for constructing a custom configuration.
    pub const fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    pub(crate) const fn to_flags(self) -> u32 {
        let mut flags = 0;
        if self.free_tone_marking {
            flags |= 1 << 0;
        }
        if self.std_tone_style {
            flags |= 1 << 1;
        }
        if self.auto_correct {
            flags |= 1 << 2;
        }
        flags
    }

    /// Creates a configuration from an integer bitmask of flags.
    ///
    /// - Bit 0 (0x01): `free_tone_marking`
    /// - Bit 1 (0x02): `std_tone_style`
    /// - Bit 2 (0x04): `auto_correct`
    pub const fn from_flags(flags: u32) -> Self {
        Self {
            free_tone_marking: (flags & (1 << 0)) != 0,
            std_tone_style: (flags & (1 << 1)) != 0,
            auto_correct: (flags & (1 << 2)) != 0,
        }
    }
}

/// A fluent builder for constructing a [`Config`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    /// Creates a new builder initialized with default settings.
    pub const fn new() -> Self {
        Self { config: Config::new() }
    }

    /// Sets whether free tone marking is allowed.
    pub const fn free_tone_marking(mut self, enabled: bool) -> Self {
        self.config.free_tone_marking = enabled;
        self
    }

    /// Sets whether standard (new) tone style is enabled (`hòa` vs `hoà`).
    pub const fn std_tone_style(mut self, enabled: bool) -> Self {
        self.config.std_tone_style = enabled;
        self
    }

    /// Sets whether automatic spelling correction is enabled.
    pub const fn auto_correct(mut self, enabled: bool) -> Self {
        self.config.auto_correct = enabled;
        self
    }

    /// Builds and returns the final [`Config`].
    pub const fn build(self) -> Config {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_flags_from_flags_roundtrip() {
        let configs = [
            Config { free_tone_marking: true, std_tone_style: true, auto_correct: true },
            Config { free_tone_marking: false, std_tone_style: false, auto_correct: false },
            Config { free_tone_marking: true, std_tone_style: false, auto_correct: true },
            Config { free_tone_marking: false, std_tone_style: true, auto_correct: false },
        ];
        for original in configs {
            let flags = original.to_flags();
            let restored = Config::from_flags(flags);
            assert_eq!(original, restored, "Round-trip failed for {original:?}");
        }
    }

    #[test]
    fn default_config_flags() {
        let cfg = Config::default();
        // Default: all three enabled -> flags = 0b111 = 7
        assert_eq!(cfg.to_flags(), 7);
    }

    #[test]
    fn config_builder() {
        let cfg = Config::builder()
            .free_tone_marking(false)
            .std_tone_style(true)
            .auto_correct(false)
            .build();

        assert_eq!(
            cfg,
            Config { free_tone_marking: false, std_tone_style: true, auto_correct: false }
        );
    }
}
