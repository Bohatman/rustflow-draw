//! Native JSON and .drawio XML serialization for RustFlow Draw.

pub const NATIVE_FORMAT_VERSION: &str = "1.0.0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_constant_is_defined() {
        assert_eq!(NATIVE_FORMAT_VERSION, "1.0.0");
    }
}
