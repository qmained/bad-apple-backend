use heatshrink::decoder::HeatshrinkDecoder;
use heatshrink::encoder::HeatshrinkEncoder;
use heatshrink::{CodecError, Poll, SinkError};
use std::fs;

fn main1() {
    // ── Encoding ──────────────────────────────────────────────────────────────────
    let input = include_bytes!("../bad_apple.bin");
    let mut decoder: HeatshrinkDecoder<10, 4, 128, 1024> = HeatshrinkDecoder::new();

    let mut output_buffer = [0u8; 1024];
    let mut buffer_size = 0;

    let mut flash_index = 0;

    while flash_index < input.len() {
        match decoder.sink(&input[flash_index..]) {
            Ok(n) => flash_index += n,
            Err(SinkError::Full) => {}
            Err(SinkError::Misuse) => panic!("Misuse"),
        }

        loop {
            let mut free_space = &mut output_buffer[buffer_size..];

            match decoder.poll(&mut free_space) {
                Ok(Poll::More(n)) => {
                    buffer_size += n;

                    if buffer_size == 1024 {
                        buffer_size = 0;
                    }
                    continue;
                }
                Ok(Poll::Empty(n)) => {
                    buffer_size += n;

                    if buffer_size == 1024 {
                        buffer_size = 0;
                    }
                    break;
                }
                Err(e) => panic!("Err: {e:?}"),
            }
        }
    }
}
pub fn main2() {
    let input: &[u8] = include_bytes!("../bad_apple_full.bin");
    let mut compressed = [0u8; 8 * 1024 * 500];
    let len = compressed.len();

    let mut encoder: HeatshrinkEncoder<10, 4, 2048> = HeatshrinkEncoder::new();

    let encoded = run_encode(&mut encoder, input, &mut compressed).unwrap();


    fs::write("output.bin", encoded).unwrap();

    let total_out = encoded.len();
    println!("{} {}", len, total_out / 1024);

}

/// Internal encode loop, generic over all encoder configurations.
pub fn run_encode<'a, const W: usize, const L: usize, const BUF: usize>(
    enc: &mut HeatshrinkEncoder<W, L, BUF>,
    src: &[u8],
    dst: &'a mut [u8],
) -> Result<&'a [u8], CodecError> {
    let mut total_input_size = 0;
    let mut total_output_size = 0;

    loop {
        if total_input_size < src.len() {
            match enc.sink(&src[total_input_size..]) {
                Ok(n) => total_input_size += n,
                Err(SinkError::Full) => {}
                Err(SinkError::Misuse) => return Err(CodecError::Internal),
            }
        }

        if total_input_size == src.len() {
            enc.finish();
        }

        if total_output_size == dst.len() {
            return Err(CodecError::OutputFull);
        }

        match enc.poll(&mut dst[total_output_size..]) {
            Ok(Poll::More(n)) => {
                total_output_size += n;
                if total_output_size == dst.len() {
                    return Err(CodecError::OutputFull);
                }
            }
            Ok(Poll::Empty(n)) => {
                total_output_size += n;
                if total_input_size == src.len() {
                    break;
                }
            }
            Err(_) => return Err(CodecError::Internal),
        }
    }

    Ok(&dst[..total_output_size])
}
