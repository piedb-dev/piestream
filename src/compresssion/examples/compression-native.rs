#![allow(dead_code)]
#![allow(unused_imports)]
// #![feature(test)]
// extern crate test;

use log::{Level, debug};

use piestream_compression::basic::Compression as CodecType;
use piestream_compression::compression::Codec;
use piestream_compression::{basic::Compression, compression::{create_codec, CodecOptionsBuilder}};
use byteorder::{ByteOrder, BigEndian};
use rand::{
    distributions::{uniform::SampleUniform, Distribution, Standard},
    thread_rng, Rng,
};

// q-compress
use q_compress::{auto_compress, auto_decompress, DEFAULT_COMPRESSION_LEVEL};

fn random_bytes(n: usize) -> Vec<u8> {
    let mut result = vec![];
    let mut rng = thread_rng();
    for _ in 0..n {
        result.push(rng.gen_range(0..255));
    }
    result
}

fn test_roundtrip(c: CodecType, data: &[u8], uncompress_size: Option<usize>) {
    debug!("test_roundtrip: \n\t codectype {:?} \n\t data {:?} \n\t uncompress_size {:?}", 
                c, data, uncompress_size);

    let codec_options = CodecOptionsBuilder::default()
        .set_backward_compatible_lz4(false)
        .build();
    let mut c1 = create_codec(c, &codec_options).unwrap().unwrap();
    let mut c2 = create_codec(c, &codec_options).unwrap().unwrap();
    // Compress with c1
    let mut compressed = Vec::new();
    let mut decompressed = Vec::new();

    debug!("test_roundtrip: start compressing data: {:?}", data);

    c1.compress(data, &mut compressed)
        .expect("Error when compressing");
    debug!("test_roundtrip: compressed: {:?}", compressed);

    // Decompress with c2
    let decompressed_size = c2
        .decompress(compressed.as_slice(), &mut decompressed, uncompress_size)
        .expect("Error when decompressing");

    debug!("test_roundtrip: decompressed: {:?}", decompressed);

    debug!("test_roundtrip: data.len {:?} decompress_size {:?}", data.len(), decompressed_size);

    assert_eq!(data.len(), decompressed_size);

    debug!("test_roundtrip: test if equal {:?} {:?} result {:?}", 
            data, decompressed.as_slice(),
            data == decompressed.as_slice());

    assert_eq!(data, decompressed.as_slice());
    decompressed.clear();
    compressed.clear();

    debug!("\n\n Now starting c2 compress\n");

    // Compress with c2
    c2.compress(data, &mut compressed)
        .expect("Error when compressing");
    // Decompress with c1
    let decompressed_size = c1
        .decompress(compressed.as_slice(), &mut decompressed, uncompress_size)
        .expect("Error when decompressing");

    debug!("\n\n Now starting c2 assert_eq! {:?} {:?}\n", data.len(), decompressed.len());

    assert_eq!(data.len(), decompressed_size);

    debug!("Now starting c2 test if equal {:?} {:?} result {:?}", 
    data, decompressed.as_slice(),
    data == decompressed.as_slice());

    assert_eq!(data, decompressed.as_slice());
    decompressed.clear();
    compressed.clear();

    if c != CodecType::QCOM {
        // Test does not trample existing data in output buffers
        let prefix = &[0xDE, 0xAD, 0xBE, 0xEF];
        decompressed.extend_from_slice(prefix);
        compressed.extend_from_slice(prefix);
        c2.compress(data, &mut compressed)
            .expect("Error when compressing");
        assert_eq!(&compressed[..4], prefix);
        let decompressed_size = c2
            .decompress(&compressed[4..], &mut decompressed, uncompress_size)
            .expect("Error when decompressing");
        assert_eq!(data.len(), decompressed_size);
        assert_eq!(data, &decompressed[4..]);
        assert_eq!(&decompressed[..4], prefix);
    }
    
}
fn test_codec_with_size(c: CodecType) {
    let sizes = vec![100, 10000, 100000];
    for size in sizes {
        let data = random_bytes(size);
        test_roundtrip(c, &data, Some(data.len()));
    }
}
fn test_codec_without_size(c: CodecType) {
    let sizes = vec![100, 10000, 100000];
    for size in sizes {
        let data = random_bytes(size);
        test_roundtrip(c, &data, None);
    }
}

// #[test]
fn test_codec_snappy() {
    test_codec_with_size(CodecType::SNAPPY);
    test_codec_without_size(CodecType::SNAPPY);
}
// #[test]
fn test_codec_gzip() {
    test_codec_with_size(CodecType::GZIP);
    test_codec_without_size(CodecType::GZIP);
}
// #[test]
fn test_codec_brotli() {
    test_codec_with_size(CodecType::BROTLI);
    test_codec_without_size(CodecType::BROTLI);
}
// #[test]
fn test_codec_lz4() {
    test_codec_with_size(CodecType::LZ4);
}
// #[test]
fn test_codec_zstd() {
    test_codec_with_size(CodecType::ZSTD);
    test_codec_without_size(CodecType::ZSTD);
}
// #[test]
fn test_codec_lz4_raw() {
    test_codec_with_size(CodecType::LZ4_RAW);
}

// #[test]
fn test_codec_qcom() {
    test_codec_with_size(CodecType::QCOM);
    test_codec_without_size(CodecType::QCOM);
}

fn main() {

    println!("\nTo enable debug output, add 'RUST_LOG=debug' before cargo run.\n");

    env_logger::init();

    test_codec_snappy();
    test_codec_gzip();
    test_codec_brotli();
    test_codec_lz4();
    test_codec_lz4_raw();
    test_codec_zstd();


    test_codec_qcom();

    println!("All examples succeed.");
}