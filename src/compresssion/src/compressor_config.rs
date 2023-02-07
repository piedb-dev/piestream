
pub const DEFAULT_COMPRESSION_LEVEL: usize = 8;

#[derive(Clone, Debug)]
pub struct CompressorConfig {

  pub compression_level: usize,

  pub delta_encoding_order: usize,

  pub use_gcds: bool,
}

impl Default for CompressorConfig {
  fn default() -> Self {
    Self {
      compression_level: DEFAULT_COMPRESSION_LEVEL,
      delta_encoding_order: 0,
      use_gcds: true,
    }
  }
}

impl CompressorConfig {

    pub fn set_compression_level(mut self, level: usize) -> Self {
        self.compression_level = level;
        self
    }

    pub fn set_delta_encoding_order(mut self, order: usize) -> Self {
        self.delta_encoding_order = order;
        self
    }

    /// Sets [`use_gcds`][CompressorConfig::use_gcds].
    pub fn set_use_gcds(mut self, use_gcds: bool) -> Self {
        self.use_gcds = use_gcds;
        self
    }
}
